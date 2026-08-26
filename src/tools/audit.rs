//! Audit log tools.

use rmcp::{
    ErrorData as McpError, handler::server::wrapper::Parameters, model::*, tool, tool_router,
};

use crate::format::keys as fmt;
use crate::helpers::with_team_fmt;
use crate::params::*;
use crate::tools::GrokManagementServer;

#[tool_router(router = router_audit, vis = "pub")]
impl GrokManagementServer {
    #[tool(
        description = "List team audit events. Optional: page_size, page_token, user_id, query, time_from, time_to."
    )]
    async fn list_audit_events(
        &self,
        Parameters(p): Parameters<ListAuditEventsParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut query = vec![("pageSize".into(), p.page_size.unwrap_or(20).to_string())];
        if let Some(tok) = p.page_token.filter(|s| !s.is_empty()) {
            query.push(("pageToken".into(), tok));
        }
        if let Some(u) = p.user_id.filter(|s| !s.is_empty()) {
            query.push(("eventFilter.userId".into(), u));
        }
        if let Some(q) = p.query.filter(|s| !s.is_empty()) {
            query.push(("eventFilter.query".into(), q));
        }
        if let Some(t) = p.time_from.filter(|s| !s.is_empty()) {
            query.push(("eventTimeFrom".into(), t));
        }
        if let Some(t) = p.time_to.filter(|s| !s.is_empty()) {
            query.push(("eventTimeTo".into(), t));
        }
        let api = self.api.clone();
        Ok(with_team_fmt(
            &self.api,
            move |tid| {
                let api = api.clone();
                async move { api.list_audit_events(&tid, &query).await }
            },
            |_tid, v| fmt::fmt_list_audit(v),
        )
        .await)
    }
}
