//! Audit log Management API methods.

use anyhow::Result;
use serde_json::Value;

use super::Api;

impl Api {
    pub async fn list_audit_events(
        &self,
        team_id: &str,
        query: &[(String, String)],
    ) -> Result<Value> {
        self.get(&format!("/audit/teams/{team_id}/events"), query)
            .await
    }
}
