use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info};

use crate::planka::PlankaClient;
use crate::tools;

use super::types::*;

pub struct McpServer {
    client: PlankaClient,
}

impl McpServer {
    pub fn new(client: PlankaClient) -> Self {
        Self { client }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        info!("MCP server started, waiting for requests...");

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;

            if bytes_read == 0 {
                info!("EOF received, shutting down");
                break;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            debug!("Received: {}", trimmed);

            let response = self.handle_message(trimmed).await;

            if let Some(resp) = response {
                let json_str = serde_json::to_string(&resp)?;
                debug!("Sending: {}", json_str);
                stdout.write_all(json_str.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }

        Ok(())
    }

    async fn handle_message(&self, msg: &str) -> Option<JsonRpcResponse> {
        let request: JsonRpcRequest = match serde_json::from_str(msg) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse request: {e}");
                return Some(JsonRpcResponse::error(None, JsonRpcError::parse_error()));
            }
        };

        // Notifications (no id) don't get responses
        if request.id.is_none() {
            self.handle_notification(&request).await;
            return None;
        }

        let result = self.handle_request(&request).await;
        Some(match result {
            Ok(value) => JsonRpcResponse::success(request.id, value),
            Err(error) => JsonRpcResponse::error(request.id, error),
        })
    }

    async fn handle_notification(&self, request: &JsonRpcRequest) {
        match request.method.as_str() {
            "notifications/initialized" => {
                info!("Client initialized");
            }
            "notifications/cancelled" => {
                debug!("Request cancelled");
            }
            _ => {
                debug!("Unknown notification: {}", request.method);
            }
        }
    }

    async fn handle_request(&self, request: &JsonRpcRequest) -> Result<Value, JsonRpcError> {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(&request.params),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tools_call(&request.params).await,
            "ping" => Ok(json!({})),
            _ => Err(JsonRpcError::method_not_found(&request.method)),
        }
    }

    fn handle_initialize(&self, _params: &Option<Value>) -> Result<Value, JsonRpcError> {
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: ToolsCapability { list_changed: false },
            },
            server_info: ServerInfo {
                name: "planka-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(e.to_string()))
    }

    fn handle_tools_list(&self) -> Result<Value, JsonRpcError> {
        let result = ToolsListResult {
            tools: tools::list_tools(),
        };

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(e.to_string()))
    }

    async fn handle_tools_call(&self, params: &Option<Value>) -> Result<Value, JsonRpcError> {
        let params: ToolCallParams = params
            .as_ref()
            .ok_or_else(|| JsonRpcError::invalid_params("Missing params"))?
            .clone()
            .try_into()
            .map_err(|_| JsonRpcError::invalid_params("Invalid params"))?;

        let result = tools::call_tool(&self.client, &params.name, params.arguments).await;

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(e.to_string()))
    }
}

impl TryFrom<Value> for ToolCallParams {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value).map_err(|_| ())
    }
}
