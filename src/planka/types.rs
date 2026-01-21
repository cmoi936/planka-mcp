use serde::{Deserialize, Serialize};
use std::fmt;

/// Card types supported by Planka
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CardType {
    Project,
    Story,
}

impl fmt::Display for CardType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CardType::Project => write!(f, "project"),
            CardType::Story => write!(f, "story"),
        }
    }
}

/// Stopwatch data for time tracking on cards
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stopwatch {
    pub started_at: Option<String>,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub position: Option<f64>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct List {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub position: Option<f64>,
    pub board_id: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: String,
    #[serde(rename = "type")]
    pub card_type: CardType,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub list_id: String,
    #[serde(default)]
    pub position: Option<f64>,
    #[serde(default)]
    pub board_id: Option<String>,
    #[serde(default)]
    pub creator_user_id: Option<String>,
    #[serde(default)]
    pub cover_attachment_id: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default, rename = "isDueDateCompleted")]
    pub is_due_completed: Option<bool>,
    #[serde(default)]
    pub stopwatch: Option<Stopwatch>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Response from GET /api/projects
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsResponse {
    pub items: Vec<Project>,
}

/// Response from GET /api/projects/{id} (includes nested boards, lists, cards)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    #[allow(dead_code)]
    pub item: Project,
    pub included: ProjectIncluded,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIncluded {
    #[serde(default)]
    pub boards: Vec<Board>,
    #[serde(default)]
    #[allow(dead_code)]
    pub lists: Vec<List>,
    #[serde(default)]
    #[allow(dead_code)]
    pub cards: Vec<Card>,
}

/// Response from GET /api/boards/{id}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardResponse {
    #[allow(dead_code)]
    pub item: Board,
    pub included: BoardIncluded,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardIncluded {
    #[serde(default)]
    pub lists: Vec<List>,
    #[serde(default)]
    pub cards: Vec<Card>,
}

/// Response from POST /api/lists/{listId}/cards
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardResponse {
    pub item: Card,
}

/// Request body for creating a card
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCardRequest {
    #[serde(rename = "type")]
    pub card_type: CardType,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub position: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "isDueDateCompleted")]
    pub is_due_completed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stopwatch: Option<Stopwatch>,
}

/// Options for creating a card
#[derive(Debug, Clone)]
pub struct CreateCardOptions {
    pub list_id: String,
    pub card_type: CardType,
    pub name: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub is_due_completed: Option<bool>,
    pub stopwatch: Option<Stopwatch>,
}

/// Response from POST /api/projects/{projectId}/boards
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardCreateResponse {
    pub item: Board,
}

/// Request body for creating a board
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBoardRequest {
    pub name: String,
    pub position: f64,
}

/// Response from POST /api/boards/{boardId}/lists
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse {
    pub item: List,
}

/// Request body for creating a list
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateListRequest {
    pub name: String,
    pub position: f64,
}

/// Options for updating a card
#[derive(Debug, Clone, Default)]
pub struct UpdateCardOptions {
    pub name: Option<String>,
    pub description: Option<String>,
    pub card_type: Option<CardType>,
    pub due_date: Option<String>,
    pub is_due_completed: Option<bool>,
    pub board_id: Option<String>,
    pub cover_attachment_id: Option<String>,
}
