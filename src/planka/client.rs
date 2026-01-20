use reqwest::{Client, header};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};
use url::Url;

use super::types::*;

#[derive(Debug, Error)]
pub enum PlankaError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("HTTP status {0}: {1}")]
    Status(u16, String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("JSON error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
enum PlankaAuth {
    Token(String),
    Credentials { email: String, password: String },
}

#[derive(Debug)]
pub struct PlankaClient {
    base_url: Url,
    http: Client,
    auth: PlankaAuth,
    cached_token: Arc<RwLock<Option<String>>>,
}

impl PlankaClient {
    pub fn from_env() -> Result<Self, PlankaError> {
        debug!("Initializing Planka client from environment variables");
        
        let base_url = std::env::var("PLANKA_URL")
            .map_err(|_| {
                error!("PLANKA_URL environment variable not set");
                PlankaError::Config("PLANKA_URL not set".into())
            })?;

        debug!(url = %base_url, "Parsing Planka base URL");
        let base_url = Url::parse(&base_url)
            .map_err(|e| {
                error!(url = %base_url, error = %e, "Invalid PLANKA_URL format");
                PlankaError::Config(format!("Invalid PLANKA_URL: {e}"))
            })?;

        let auth = if let Ok(token) = std::env::var("PLANKA_TOKEN") {
            debug!("Using token-based authentication");
            PlankaAuth::Token(token)
        } else {
            debug!("Using email/password authentication");
            let email = std::env::var("PLANKA_EMAIL")
                .map_err(|_| {
                    error!("Neither PLANKA_TOKEN nor PLANKA_EMAIL is set");
                    PlankaError::Config("PLANKA_TOKEN or PLANKA_EMAIL must be set".into())
                })?;
            let password = std::env::var("PLANKA_PASSWORD")
                .map_err(|_| {
                    error!("PLANKA_PASSWORD not set but PLANKA_EMAIL is configured");
                    PlankaError::Config("PLANKA_PASSWORD must be set when using PLANKA_EMAIL".into())
                })?;
            PlankaAuth::Credentials { email, password }
        };

        let http = Client::builder()
            .build()
            .map_err(|e| {
                error!(error = %e, "Failed to build HTTP client");
                PlankaError::Http(e)
            })?;

        info!(base_url = %base_url, "Planka client configured successfully");
        Ok(Self {
            base_url,
            http,
            auth,
            cached_token: Arc::new(RwLock::new(None)),
        })
    }

    async fn get_token(&self) -> Result<String, PlankaError> {
        match &self.auth {
            PlankaAuth::Token(token) => {
                trace!("Using configured bearer token");
                Ok(token.clone())
            }
            PlankaAuth::Credentials { email, password } => {
                // Check cache first
                {
                    let cache = self.cached_token.read().await;
                    if let Some(token) = cache.as_ref() {
                        trace!("Using cached authentication token");
                        return Ok(token.clone());
                    }
                }

                // Fetch new token
                info!(email = %email, "Authenticating with Planka API");
                let url = self.base_url.join("/api/access-tokens")?;

                trace!(url = %url, "Sending authentication request");
                let resp = self.http
                    .post(url.clone())
                    .json(&serde_json::json!({
                        "emailOrUsername": email,
                        "password": password
                    }))
                    .send()
                    .await
                    .map_err(|e| {
                        error!(
                            url = %url,
                            error = %e,
                            "Failed to send authentication request"
                        );
                        e
                    })?;

                let status = resp.status();
                if !status.is_success() {
                    let status_code = status.as_u16();
                    let body = resp.text().await.unwrap_or_default();
                    error!(
                        status = status_code,
                        response_body = %body,
                        "Authentication failed"
                    );
                    return Err(PlankaError::Status(status_code, body));
                }

                let data: serde_json::Value = resp.json().await.map_err(|e| {
                    error!(error = %e, "Failed to parse authentication response");
                    e
                })?;
                
                trace!(response = ?data, "Authentication response received");
                
                let token = data["item"]
                    .as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| {
                        error!("No token in authentication response");
                        PlankaError::Config("No token in login response".into())
                    })?;

                // Cache the token
                {
                    let mut cache = self.cached_token.write().await;
                    *cache = Some(token.clone());
                }

                info!("Authentication successful, token cached");
                Ok(token)
            }
        }
    }

    async fn request(&self, method: reqwest::Method, path: &str) -> Result<reqwest::RequestBuilder, PlankaError> {
        trace!(method = %method, path = %path, "Preparing API request");
        let token = self.get_token().await?;
        let url = self.base_url.join(path)?;

        debug!(method = %method, url = %url, "Building API request");
        Ok(self.http
            .request(method, url)
            .header(header::AUTHORIZATION, format!("Bearer {token}")))
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>, PlankaError> {
        debug!("Listing all projects");
        let resp = self.request(reqwest::Method::GET, "/api/projects")
            .await?
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, path = "/api/projects", "Failed to send request");
                e
            })?;

        let status = resp.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            error!(
                status = status_code,
                path = "/api/projects",
                response_body = %body,
                "API request failed"
            );
            return Err(PlankaError::Status(status_code, body));
        }

        let data: ProjectsResponse = resp.json().await.map_err(|e| {
            error!(error = %e, path = "/api/projects", "Failed to parse response JSON");
            e
        })?;
        
        info!(count = data.items.len(), "Successfully listed projects");
        trace!(projects = ?data.items, "Project details");
        Ok(data.items)
    }

    pub async fn list_boards(&self, project_id: &str) -> Result<Vec<Board>, PlankaError> {
        debug!(project_id = %project_id, "Listing boards for project");
        let path = format!("/api/projects/{project_id}");
        let resp = self.request(reqwest::Method::GET, &path)
            .await?
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, path = %path, "Failed to send request");
                e
            })?;

        let status = resp.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            error!(
                status = status_code,
                path = %path,
                response_body = %body,
                "API request failed"
            );
            return Err(PlankaError::Status(status_code, body));
        }

        let data: ProjectResponse = resp.json().await.map_err(|e| {
            error!(error = %e, path = %path, "Failed to parse response JSON");
            e
        })?;
        
        info!(project_id = %project_id, count = data.included.boards.len(), "Successfully listed boards");
        trace!(boards = ?data.included.boards, "Board details");
        Ok(data.included.boards)
    }

    pub async fn list_cards(&self, board_id: &str) -> Result<Vec<Card>, PlankaError> {
        debug!(board_id = %board_id, "Listing cards for board");
        let path = format!("/api/boards/{board_id}");
        let resp = self.request(reqwest::Method::GET, &path)
            .await?
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, path = %path, "Failed to send request");
                e
            })?;

        let status = resp.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            error!(
                status = status_code,
                path = %path,
                response_body = %body,
                "API request failed"
            );
            return Err(PlankaError::Status(status_code, body));
        }

        let data: BoardResponse = resp.json().await.map_err(|e| {
            error!(error = %e, path = %path, "Failed to parse response JSON");
            e
        })?;
        
        info!(board_id = %board_id, count = data.included.cards.len(), "Successfully listed cards");
        trace!(cards = ?data.included.cards, "Card details");
        Ok(data.included.cards)
    }

    pub async fn list_lists(&self, board_id: &str) -> Result<Vec<List>, PlankaError> {
        debug!(board_id = %board_id, "Listing lists for board");
        let path = format!("/api/boards/{board_id}");
        let resp = self.request(reqwest::Method::GET, &path)
            .await?
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, path = %path, "Failed to send request");
                e
            })?;

        let status = resp.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            error!(
                status = status_code,
                path = %path,
                response_body = %body,
                "API request failed"
            );
            return Err(PlankaError::Status(status_code, body));
        }

        let data: BoardResponse = resp.json().await.map_err(|e| {
            error!(error = %e, path = %path, "Failed to parse response JSON");
            e
        })?;
        
        info!(board_id = %board_id, count = data.included.lists.len(), "Successfully listed lists");
        trace!(lists = ?data.included.lists, "List details");
        Ok(data.included.lists)
    }

    pub async fn create_card(&self, options: CreateCardOptions) -> Result<Card, PlankaError> {
        info!(
            list_id = %options.list_id,
            card_type = %options.card_type,
            name = %options.name,
            "Creating new card"
        );
        trace!(options = ?options, "Card creation options");
        
        let path = format!("/api/lists/{}/cards", options.list_id);

        let body = CreateCardRequest {
            card_type: options.card_type,
            name: options.name,
            description: options.description,
            position: 65535.0, // Default position at end
            due_date: options.due_date,
            is_due_completed: options.is_due_completed,
            stopwatch: options.stopwatch,
        };

        trace!(request_body = ?body, "Card creation request");

        let resp = self.request(reqwest::Method::POST, &path)
            .await?
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, path = %path, "Failed to send card creation request");
                e
            })?;

        let status = resp.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            error!(
                status = status_code,
                path = %path,
                response_body = %body,
                "Card creation failed"
            );
            return Err(PlankaError::Status(status_code, body));
        }

        let data: CardResponse = resp.json().await.map_err(|e| {
            error!(error = %e, path = %path, "Failed to parse card creation response");
            e
        })?;
        
        info!(card_id = %data.item.id, "Card created successfully");
        trace!(card = ?data.item, "Created card details");
        Ok(data.item)
    }

    pub async fn create_board(
        &self,
        project_id: &str,
        name: &str,
    ) -> Result<Board, PlankaError> {
        info!(project_id = %project_id, name = %name, "Creating new board");
        let path = format!("/api/projects/{project_id}/boards");

        let body = CreateBoardRequest {
            name: name.to_string(),
            position: 65535.0,
        };

        trace!(request_body = ?body, "Board creation request");

        let resp = self.request(reqwest::Method::POST, &path)
            .await?
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, path = %path, "Failed to send board creation request");
                e
            })?;

        let status = resp.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            error!(
                status = status_code,
                path = %path,
                response_body = %body,
                "Board creation failed"
            );
            return Err(PlankaError::Status(status_code, body));
        }

        let data: BoardCreateResponse = resp.json().await.map_err(|e| {
            error!(error = %e, path = %path, "Failed to parse board creation response");
            e
        })?;
        
        info!(board_id = %data.item.id, "Board created successfully");
        trace!(board = ?data.item, "Created board details");
        Ok(data.item)
    }

    pub async fn create_list(
        &self,
        board_id: &str,
        name: &str,
    ) -> Result<List, PlankaError> {
        info!(board_id = %board_id, name = %name, "Creating new list");
        let path = format!("/api/boards/{board_id}/lists");

        let body = CreateListRequest {
            name: name.to_string(),
            position: 65535.0,
        };

        trace!(request_body = ?body, "List creation request");

        let resp = self.request(reqwest::Method::POST, &path)
            .await?
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, path = %path, "Failed to send list creation request");
                e
            })?;

        let status = resp.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            error!(
                status = status_code,
                path = %path,
                response_body = %body,
                "List creation failed"
            );
            return Err(PlankaError::Status(status_code, body));
        }

        let data: ListResponse = resp.json().await.map_err(|e| {
            error!(error = %e, path = %path, "Failed to parse list creation response");
            e
        })?;
        
        info!(list_id = %data.item.id, "List created successfully");
        trace!(list = ?data.item, "Created list details");
        Ok(data.item)
    }

    pub async fn update_card(
        &self,
        card_id: &str,
        options: UpdateCardOptions,
    ) -> Result<Card, PlankaError> {
        info!(card_id = %card_id, "Updating card");
        trace!(options = ?options, "Card update options");
        
        let path = format!("/api/cards/{card_id}");

        let mut body = serde_json::Map::new();
        if let Some(n) = options.name {
            body.insert("name".to_string(), serde_json::Value::String(n));
        }
        if let Some(d) = options.description {
            body.insert("description".to_string(), serde_json::Value::String(d));
        }
        if let Some(t) = options.card_type {
            body.insert("type".to_string(), serde_json::Value::String(t.to_string()));
        }
        if let Some(dd) = options.due_date {
            body.insert("dueDate".to_string(), serde_json::Value::String(dd));
        }
        if let Some(dc) = options.is_due_completed {
            body.insert("isDueCompleted".to_string(), serde_json::Value::Bool(dc));
        }
        if let Some(bid) = options.board_id {
            body.insert("boardId".to_string(), serde_json::Value::String(bid));
        }
        if let Some(cid) = options.cover_attachment_id {
            body.insert("coverAttachmentId".to_string(), serde_json::Value::String(cid));
        }

        trace!(request_body = ?body, "Card update request");

        let resp = self.request(reqwest::Method::PATCH, &path)
            .await?
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, path = %path, "Failed to send card update request");
                e
            })?;

        let status = resp.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            error!(
                status = status_code,
                path = %path,
                response_body = %body,
                "Card update failed"
            );
            return Err(PlankaError::Status(status_code, body));
        }

        let data: CardResponse = resp.json().await.map_err(|e| {
            error!(error = %e, path = %path, "Failed to parse card update response");
            e
        })?;
        
        info!(card_id = %card_id, "Card updated successfully");
        trace!(card = ?data.item, "Updated card details");
        Ok(data.item)
    }

    pub async fn move_card(
        &self,
        card_id: &str,
        list_id: &str,
        position: Option<f64>,
    ) -> Result<Card, PlankaError> {
        info!(card_id = %card_id, list_id = %list_id, position = ?position, "Moving card");
        let path = format!("/api/cards/{card_id}");

        let mut body = serde_json::Map::new();
        body.insert("listId".to_string(), serde_json::Value::String(list_id.to_string()));
        let pos = position.unwrap_or(65535.0);
        body.insert("position".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(pos).unwrap()));

        trace!(request_body = ?body, "Card move request");

        let resp = self.request(reqwest::Method::PATCH, &path)
            .await?
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, path = %path, "Failed to send card move request");
                e
            })?;

        let status = resp.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            error!(
                status = status_code,
                path = %path,
                response_body = %body,
                "Card move failed"
            );
            return Err(PlankaError::Status(status_code, body));
        }

        let data: CardResponse = resp.json().await.map_err(|e| {
            error!(error = %e, path = %path, "Failed to parse card move response");
            e
        })?;
        
        info!(card_id = %card_id, new_list_id = %list_id, "Card moved successfully");
        trace!(card = ?data.item, "Moved card details");
        Ok(data.item)
    }

    pub async fn delete_card(&self, card_id: &str) -> Result<(), PlankaError> {
        warn!(card_id = %card_id, "Deleting card");
        let path = format!("/api/cards/{card_id}");

        let resp = self.request(reqwest::Method::DELETE, &path)
            .await?
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, path = %path, "Failed to send card deletion request");
                e
            })?;

        let status = resp.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            error!(
                status = status_code,
                path = %path,
                response_body = %body,
                "Card deletion failed"
            );
            return Err(PlankaError::Status(status_code, body));
        }

        info!(card_id = %card_id, "Card deleted successfully");
        Ok(())
    }

    pub async fn delete_list(&self, list_id: &str) -> Result<(), PlankaError> {
        warn!(list_id = %list_id, "Deleting list and all its cards");
        let path = format!("/api/lists/{list_id}");

        let resp = self.request(reqwest::Method::DELETE, &path)
            .await?
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, path = %path, "Failed to send list deletion request");
                e
            })?;

        let status = resp.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            error!(
                status = status_code,
                path = %path,
                response_body = %body,
                "List deletion failed"
            );
            return Err(PlankaError::Status(status_code, body));
        }

        info!(list_id = %list_id, "List deleted successfully");
        Ok(())
    }
}

impl From<url::ParseError> for PlankaError {
    fn from(e: url::ParseError) -> Self {
        PlankaError::Config(format!("URL parse error: {e}"))
    }
}
