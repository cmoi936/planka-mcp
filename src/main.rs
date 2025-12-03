mod mcp;
mod planka;
mod tools;

use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use mcp::McpServer;
use planka::PlankaClient;

#[tokio::main]
async fn main() {
    // Initialize logging (writes to stderr to keep stdout clean for JSON-RPC)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    info!("Starting planka-mcp server v{}", env!("CARGO_PKG_VERSION"));

    let client = match PlankaClient::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to initialize Planka client: {e}");
            std::process::exit(1);
        }
    };

    let server = McpServer::new(client);

    if let Err(e) = server.run().await {
        error!("Server error: {e}");
        std::process::exit(1);
    }
}
