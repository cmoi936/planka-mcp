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
    // Log level can be configured via RUST_LOG environment variable:
    // - RUST_LOG=error (only errors)
    // - RUST_LOG=warn  (warnings and errors)
    // - RUST_LOG=info  (info, warnings, and errors) - default
    // - RUST_LOG=debug (debug, info, warnings, and errors)
    // - RUST_LOG=trace (all log messages including request/response details)
    // 
    // Examples:
    // - RUST_LOG=planka_mcp=debug (debug level for planka_mcp only)
    // - RUST_LOG=planka_mcp::planka=trace (trace level for API client)
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("planka_mcp=info"));
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(true)
        .with_level(true)
        .init();

    info!(
        version = %env!("CARGO_PKG_VERSION"),
        "Starting planka-mcp server"
    );

    let client = match PlankaClient::from_env() {
        Ok(c) => {
            info!("Planka client initialized successfully");
            c
        }
        Err(e) => {
            error!(
                error = %e,
                "Failed to initialize Planka client"
            );
            std::process::exit(1);
        }
    };

    let server = McpServer::new(client);

    info!("MCP server initialized, starting event loop");
    if let Err(e) = server.run().await {
        error!(
            error = %e,
            "Server error occurred"
        );
        std::process::exit(1);
    }
    
    info!("Server shutdown complete");
}
