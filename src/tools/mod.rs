//! MCP tools composed from domain routers (no god impl).

mod audit;
mod billing;
mod keys;

use rmcp::{ServerHandler, handler::server::tool::ToolRouter, model::*, tool_handler};
use std::sync::Arc;

use crate::api::Api;
use crate::config::Config;

/// MCP server state: shared API client only. Tools live in domain modules.
#[derive(Clone)]
pub struct GrokManagementServer {
    pub(crate) api: Arc<Api>,
}

impl GrokManagementServer {
    pub fn new(config: Config) -> Self {
        Self {
            api: Arc::new(Api::new(&config)),
        }
    }

    /// Combined tool surface for `#[tool_handler]`.
    pub fn tool_router() -> ToolRouter<Self> {
        Self::router_keys() + Self::router_billing() + Self::router_audit()
    }
}

#[tool_handler]
impl ServerHandler for GrokManagementServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "grok-management",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "xAI Management API — API keys + billing. Prefer billing_overview for spend.                  Gates: DELETE, ROTATE, YES-WRITE, TOP-UP. Amounts in USD cents.                  BillingWrite required for write billing tools.",
            )
    }
}
