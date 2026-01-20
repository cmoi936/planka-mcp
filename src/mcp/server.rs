use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info, trace, warn};

use crate::planka::PlankaClient;
use crate::tools;

use super::types::*;

pub struct McpServer {
    client: PlankaClient,
}

impl McpServer {
    pub fn new(client: PlankaClient) -> Self {
        info!("MCP server instance created");
        Self { client }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        info!("MCP server event loop started, waiting for JSON-RPC requests on stdin");

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;

            if bytes_read == 0 {
                info!("EOF received on stdin, shutting down server");
                break;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                trace!("Received empty line, skipping");
                continue;
            }

            debug!(message_length = trimmed.len(), "Received JSON-RPC request");
            trace!(request = %trimmed, "Raw request content");

            let response = self.handle_message(trimmed).await;

            if let Some(resp) = response {
                let json_str = serde_json::to_string(&resp)?;
                trace!(response = %json_str, "Sending JSON-RPC response");
                stdout.write_all(json_str.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
                debug!("Response sent successfully");
            } else {
                trace!("No response required (notification)");
            }
        }

        info!("MCP server event loop terminated");
        Ok(())
    }

    async fn handle_message(&self, msg: &str) -> Option<JsonRpcResponse> {
        let request: JsonRpcRequest = match serde_json::from_str::<JsonRpcRequest>(msg) {
            Ok(req) => {
                debug!(
                    method = %req.method,
                    id = ?req.id,
                    "Parsed JSON-RPC request"
                );
                req
            }
            Err(e) => {
                error!(
                    error = %e,
                    message = %msg,
                    "Failed to parse JSON-RPC request"
                );
                return Some(JsonRpcResponse::error(None, JsonRpcError::parse_error()));
            }
        };

        // Notifications (no id) don't get responses
        if request.id.is_none() {
            debug!(method = %request.method, "Handling notification (no response)");
            self.handle_notification(&request).await;
            return None;
        }

        let result = self.handle_request(&request).await;
        Some(match result {
            Ok(value) => {
                debug!(id = ?request.id, "Request handled successfully");
                trace!(result = ?value, "Request result");
                JsonRpcResponse::success(request.id, value)
            }
            Err(error) => {
                warn!(
                    id = ?request.id,
                    error_code = error.code,
                    error_message = %error.message,
                    "Request failed with error"
                );
                JsonRpcResponse::error(request.id, error)
            }
        })
    }

    async fn handle_notification(&self, request: &JsonRpcRequest) {
        match request.method.as_str() {
            "notifications/initialized" => {
                info!("Client sent initialized notification");
            }
            "notifications/cancelled" => {
                info!("Client cancelled a request");
                trace!(request = ?request, "Cancellation details");
            }
            _ => {
                warn!(method = %request.method, "Unknown notification method");
                trace!(notification = ?request, "Unknown notification details");
            }
        }
    }

    async fn handle_request(&self, request: &JsonRpcRequest) -> Result<Value, JsonRpcError> {
        debug!(method = %request.method, "Dispatching request to handler");
        match request.method.as_str() {
            "initialize" => {
                info!("Handling initialize request");
                self.handle_initialize(&request.params)
            }
            "tools/list" => {
                info!("Handling tools/list request");
                self.handle_tools_list()
            }
            "tools/call" => {
                info!("Handling tools/call request");
                self.handle_tools_call(&request.params).await
            }
            "ping" => {
                debug!("Handling ping request");
                Ok(json!({}))
            }
            _ => {
                warn!(method = %request.method, "Unknown method");
                Err(JsonRpcError::method_not_found(&request.method))
            }
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

        info!(
            version = %result.server_info.version,
            protocol = %result.protocol_version,
            "Initialization complete"
        );

        serde_json::to_value(result).map_err(|e| {
            error!(error = %e, "Failed to serialize initialize result");
            JsonRpcError::internal_error(e.to_string())
        })
    }

    fn handle_tools_list(&self) -> Result<Value, JsonRpcError> {
        let tools = tools::list_tools();
        info!(tool_count = tools.len(), "Returning tools list");
        trace!(tools = ?tools, "Available tools");
        
        let result = ToolsListResult { tools };

        serde_json::to_value(result).map_err(|e| {
            error!(error = %e, "Failed to serialize tools list");
            JsonRpcError::internal_error(e.to_string())
        })
    }

    async fn handle_tools_call(&self, params: &Option<Value>) -> Result<Value, JsonRpcError> {
        let params: ToolCallParams = params
            .as_ref()
            .ok_or_else(|| {
                error!("tools/call request missing params");
                JsonRpcError::invalid_params("Missing params")
            })?
            .clone()
            .try_into()
            .map_err(|_| {
                error!(params = ?params, "Invalid tools/call params");
                JsonRpcError::invalid_params("Invalid params")
            })?;

        info!(tool = %params.name, "Calling tool");
        trace!(tool_args = ?params.arguments, "Tool arguments");

        let result = tools::call_tool(&self.client, &params.name, params.arguments).await;

        trace!(tool_result = ?result, "Tool execution result");

        serde_json::to_value(result).map_err(|e| {
            error!(error = %e, tool = %params.name, "Failed to serialize tool result");
            JsonRpcError::internal_error(e.to_string())
        })
    }
}

impl TryFrom<Value> for ToolCallParams {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value).map_err(|_| ())
    }
}
