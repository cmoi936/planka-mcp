use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, error, info, trace, warn};

use crate::mcp::types::{Tool, ToolAnnotations, ToolCallResult};
use crate::planka::PlankaClient;

/// Creates annotations enabling programmatic tool calling
fn programmatic_annotations() -> Option<ToolAnnotations> {
    Some(ToolAnnotations {
        allowed_callers: Some(vec!["code_execution_20250825".to_string()]),
    })
}

/// Returns the list of available tools
pub fn list_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_projects".to_string(),
            description: "List all Planka projects".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "list_boards".to_string(),
            description: "List all boards in a project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "The project ID"
                    }
                },
                "required": ["project_id"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "list_lists".to_string(),
            description: "List all lists (columns) on a board".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {
                        "type": "string",
                        "description": "The board ID"
                    }
                },
                "required": ["board_id"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "list_cards".to_string(),
            description: "List all cards on a board".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {
                        "type": "string",
                        "description": "The board ID"
                    }
                },
                "required": ["board_id"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "create_board".to_string(),
            description: "Create a new board in a project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "The project ID to create the board in"
                    },
                    "name": {
                        "type": "string",
                        "description": "The board name"
                    }
                },
                "required": ["project_id", "name"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "create_list".to_string(),
            description: "Create a new list (column) on a board".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {
                        "type": "string",
                        "description": "The board ID to create the list on"
                    },
                    "name": {
                        "type": "string",
                        "description": "The list name"
                    }
                },
                "required": ["board_id", "name"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "create_card".to_string(),
            description: "Create a new card in a list".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "list_id": {
                        "type": "string",
                        "description": "The list ID to create the card in"
                    },
                    "type": {
                        "type": "string",
                        "enum": ["project", "story"],
                        "description": "Type of the card (project or story)",
                        "default": "project"
                    },
                    "name": {
                        "type": "string",
                        "description": "The card title"
                    },
                    "description": {
                        "type": "string",
                        "description": "Optional card description"
                    },
                    "due_date": {
                        "type": "string",
                        "format": "date-time",
                        "description": "Optional due date (ISO 8601 format)"
                    },
                    "is_due_completed": {
                        "type": "boolean",
                        "description": "Whether the due date is completed"
                    }
                },
                "required": ["list_id", "name"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "update_card".to_string(),
            description: "Update a card's properties (name, description, type, due date, etc.)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "card_id": {
                        "type": "string",
                        "description": "The card ID to update"
                    },
                    "name": {
                        "type": "string",
                        "description": "New card title (optional)"
                    },
                    "description": {
                        "type": "string",
                        "description": "New card description (optional)"
                    },
                    "type": {
                        "type": "string",
                        "enum": ["project", "story"],
                        "description": "Card type (optional)"
                    },
                    "due_date": {
                        "type": "string",
                        "format": "date-time",
                        "description": "Due date in ISO 8601 format (optional)"
                    },
                    "is_due_completed": {
                        "type": "boolean",
                        "description": "Whether the due date is completed (optional)"
                    },
                    "board_id": {
                        "type": "string",
                        "description": "Move card to different board (optional)"
                    },
                    "cover_attachment_id": {
                        "type": "string",
                        "description": "Set cover image attachment ID (optional)"
                    }
                },
                "required": ["card_id"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "move_card".to_string(),
            description: "Move a card to a different list".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "card_id": {
                        "type": "string",
                        "description": "The card ID to move"
                    },
                    "list_id": {
                        "type": "string",
                        "description": "The target list ID"
                    },
                    "position": {
                        "type": "number",
                        "description": "Position in the list (optional)"
                    }
                },
                "required": ["card_id", "list_id"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "delete_card".to_string(),
            description: "Delete a card".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "card_id": {
                        "type": "string",
                        "description": "The card ID to delete"
                    }
                },
                "required": ["card_id"]
            }),
            // Not enabled for programmatic calling (destructive operation)
            annotations: None,
        },
        Tool {
            name: "delete_list".to_string(),
            description: "Delete a list and all its cards".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "list_id": {
                        "type": "string",
                        "description": "The list ID to delete"
                    }
                },
                "required": ["list_id"]
            }),
            // Not enabled for programmatic calling (destructive operation)
            annotations: None,
        },
    ]
}

/// Dispatch a tool call to the appropriate handler
pub async fn call_tool(client: &PlankaClient, name: &str, args: Option<Value>) -> ToolCallResult {
    debug!(tool = %name, "Dispatching tool call");
    trace!(tool = %name, args = ?args, "Tool call arguments");
    
    let result = match name {
        "list_projects" => list_projects(client).await,
        "list_boards" => list_boards(client, args).await,
        "list_lists" => list_lists(client, args).await,
        "list_cards" => list_cards(client, args).await,
        "create_board" => create_board(client, args).await,
        "create_list" => create_list(client, args).await,
        "create_card" => create_card(client, args).await,
        "update_card" => update_card(client, args).await,
        "move_card" => move_card(client, args).await,
        "delete_card" => delete_card(client, args).await,
        "delete_list" => delete_list(client, args).await,
        _ => {
            error!(tool = %name, "Unknown tool requested");
            ToolCallResult::error(format!("Unknown tool: {name}"))
        }
    };
    
    match &result {
        ToolCallResult { is_error: Some(true), .. } => {
            warn!(tool = %name, "Tool call failed");
            trace!(tool = %name, result = ?result, "Tool failure details");
        }
        _ => {
            info!(tool = %name, "Tool call succeeded");
            trace!(tool = %name, result = ?result, "Tool success details");
        }
    }
    
    result
}

async fn list_projects(client: &PlankaClient) -> ToolCallResult {
    debug!("Executing list_projects tool");
    match client.list_projects().await {
        Ok(projects) => {
            info!(count = projects.len(), "Projects listed successfully");
            let json = serde_json::to_string_pretty(&projects).unwrap_or_default();
            ToolCallResult::text(json)
        }
        Err(e) => {
            error!(error = %e, "Failed to list projects");
            ToolCallResult::error(format!("Failed to list projects: {e}"))
        }
    }
}

#[derive(Deserialize)]
struct ListBoardsArgs {
    project_id: String,
}

async fn list_boards(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: ListBoardsArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required argument: project_id"),
    };

    match client.list_boards(&args.project_id).await {
        Ok(boards) => {
            let json = serde_json::to_string_pretty(&boards).unwrap_or_default();
            ToolCallResult::text(json)
        }
        Err(e) => ToolCallResult::error(format!("Failed to list boards: {e}")),
    }
}

#[derive(Deserialize)]
struct ListListsArgs {
    board_id: String,
}

async fn list_lists(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: ListListsArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required argument: board_id"),
    };

    match client.list_lists(&args.board_id).await {
        Ok(lists) => {
            let json = serde_json::to_string_pretty(&lists).unwrap_or_default();
            ToolCallResult::text(json)
        }
        Err(e) => ToolCallResult::error(format!("Failed to list lists: {e}")),
    }
}

#[derive(Deserialize)]
struct ListCardsArgs {
    board_id: String,
}

async fn list_cards(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: ListCardsArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required argument: board_id"),
    };

    match client.list_cards(&args.board_id).await {
        Ok(cards) => {
            let json = serde_json::to_string_pretty(&cards).unwrap_or_default();
            ToolCallResult::text(json)
        }
        Err(e) => ToolCallResult::error(format!("Failed to list cards: {e}")),
    }
}

#[derive(Deserialize)]
struct CreateBoardArgs {
    project_id: String,
    name: String,
}

async fn create_board(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: CreateBoardArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required arguments: project_id, name"),
    };

    match client.create_board(&args.project_id, &args.name).await {
        Ok(board) => {
            let json = serde_json::to_string_pretty(&board).unwrap_or_default();
            ToolCallResult::text(json)
        }
        Err(e) => ToolCallResult::error(format!("Failed to create board: {e}")),
    }
}

#[derive(Deserialize)]
struct CreateListArgs {
    board_id: String,
    name: String,
}

async fn create_list(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: CreateListArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required arguments: board_id, name"),
    };

    match client.create_list(&args.board_id, &args.name).await {
        Ok(list) => {
            let json = serde_json::to_string_pretty(&list).unwrap_or_default();
            ToolCallResult::text(json)
        }
        Err(e) => ToolCallResult::error(format!("Failed to create list: {e}")),
    }
}

#[derive(Deserialize)]
struct CreateCardArgs {
    list_id: String,
    #[serde(rename = "type", default = "default_card_type")]
    card_type: String,
    name: String,
    description: Option<String>,
    due_date: Option<String>,
    is_due_completed: Option<bool>,
}

fn default_card_type() -> String {
    "project".to_string()
}

async fn create_card(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: CreateCardArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required arguments: list_id, name"),
    };

    // Parse card type
    use crate::planka::types::{CardType, CreateCardOptions};
    let card_type = match args.card_type.to_lowercase().as_str() {
        "project" => CardType::Project,
        "story" => CardType::Story,
        _ => return ToolCallResult::error("Invalid card type. Must be 'project' or 'story'"),
    };

    let options = CreateCardOptions {
        list_id: args.list_id,
        card_type,
        name: args.name,
        description: args.description,
        due_date: args.due_date,
        is_due_completed: args.is_due_completed,
        stopwatch: None, // stopwatch not supported in tool args for now
    };

    match client.create_card(options).await {
        Ok(card) => {
            let json = serde_json::to_string_pretty(&card).unwrap_or_default();
            ToolCallResult::text(json)
        }
        Err(e) => ToolCallResult::error(format!("Failed to create card: {e}")),
    }
}

#[derive(Deserialize)]
struct UpdateCardArgs {
    card_id: String,
    name: Option<String>,
    description: Option<String>,
    #[serde(rename = "type")]
    card_type: Option<String>,
    due_date: Option<String>,
    is_due_completed: Option<bool>,
    board_id: Option<String>,
    cover_attachment_id: Option<String>,
}

async fn update_card(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: UpdateCardArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required argument: card_id"),
    };

    // Parse card type if provided
    use crate::planka::types::{CardType, UpdateCardOptions};
    let card_type = if let Some(ref type_str) = args.card_type {
        match type_str.to_lowercase().as_str() {
            "project" => Some(CardType::Project),
            "story" => Some(CardType::Story),
            _ => return ToolCallResult::error("Invalid card type. Must be 'project' or 'story'"),
        }
    } else {
        None
    };

    let options = UpdateCardOptions {
        name: args.name,
        description: args.description,
        card_type,
        due_date: args.due_date,
        is_due_completed: args.is_due_completed,
        board_id: args.board_id,
        cover_attachment_id: args.cover_attachment_id,
    };

    match client.update_card(&args.card_id, options).await {
        Ok(card) => {
            let json = serde_json::to_string_pretty(&card).unwrap_or_default();
            ToolCallResult::text(json)
        }
        Err(e) => ToolCallResult::error(format!("Failed to update card: {e}")),
    }
}

#[derive(Deserialize)]
struct MoveCardArgs {
    card_id: String,
    list_id: String,
    position: Option<f64>,
}

async fn move_card(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: MoveCardArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required arguments: card_id, list_id"),
    };

    match client.move_card(&args.card_id, &args.list_id, args.position).await {
        Ok(card) => {
            let json = serde_json::to_string_pretty(&card).unwrap_or_default();
            ToolCallResult::text(json)
        }
        Err(e) => ToolCallResult::error(format!("Failed to move card: {e}")),
    }
}

#[derive(Deserialize)]
struct DeleteCardArgs {
    card_id: String,
}

async fn delete_card(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: DeleteCardArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required argument: card_id"),
    };

    match client.delete_card(&args.card_id).await {
        Ok(()) => ToolCallResult::text("Card deleted successfully"),
        Err(e) => ToolCallResult::error(format!("Failed to delete card: {e}")),
    }
}

#[derive(Deserialize)]
struct DeleteListArgs {
    list_id: String,
}

async fn delete_list(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: DeleteListArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required argument: list_id"),
    };

    match client.delete_list(&args.list_id).await {
        Ok(()) => ToolCallResult::text("List deleted successfully"),
        Err(e) => ToolCallResult::error(format!("Failed to delete list: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tools_returns_all_tools() {
        let tools = list_tools();
        assert_eq!(tools.len(), 11, "Expected 11 tools");

        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"list_projects"));
        assert!(names.contains(&"list_boards"));
        assert!(names.contains(&"list_lists"));
        assert!(names.contains(&"list_cards"));
        assert!(names.contains(&"create_board"));
        assert!(names.contains(&"create_list"));
        assert!(names.contains(&"create_card"));
        assert!(names.contains(&"update_card"));
        assert!(names.contains(&"move_card"));
        assert!(names.contains(&"delete_card"));
        assert!(names.contains(&"delete_list"));
    }

    #[test]
    fn test_programmatic_tools_have_allowed_callers() {
        let tools = list_tools();
        let programmatic_tools = [
            "list_projects",
            "list_boards",
            "list_lists",
            "list_cards",
            "create_board",
            "create_list",
            "create_card",
            "update_card",
            "move_card",
        ];

        for tool_name in programmatic_tools {
            let tool = tools.iter().find(|t| t.name == tool_name).unwrap();
            let annotations = tool
                .annotations
                .as_ref()
                .unwrap_or_else(|| panic!("{tool_name} should have annotations"));
            let callers = annotations
                .allowed_callers
                .as_ref()
                .unwrap_or_else(|| panic!("{tool_name} should have allowed_callers"));
            assert!(
                callers.contains(&"code_execution_20250825".to_string()),
                "{tool_name} should allow code_execution_20250825"
            );
        }
    }

    #[test]
    fn test_delete_tools_excluded_from_programmatic_calling() {
        let tools = list_tools();
        let delete_tools = ["delete_card", "delete_list"];

        for tool_name in delete_tools {
            let tool = tools.iter().find(|t| t.name == tool_name).unwrap();
            assert!(
                tool.annotations.is_none(),
                "{tool_name} should NOT have annotations (destructive operation)"
            );
        }
    }
}
