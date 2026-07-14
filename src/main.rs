mod api;
mod config;
mod format;
mod helpers;
mod params;
mod tools;
mod util;

use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing::info;
use tracing_subscriber::EnvFilter;

use tools::GrokManagementServer;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    info!("loading xAI Management API config");
    let config = config::load()?;

    info!("starting mcp-server-grok-management");
    let server = GrokManagementServer::new(config);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
