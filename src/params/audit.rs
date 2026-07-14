//! Audit tool parameters.

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListAuditEventsParams {
    pub page_size: Option<u32>,
    pub page_token: Option<String>,
    pub user_id: Option<String>,
    pub query: Option<String>,
    pub time_from: Option<String>,
    pub time_to: Option<String>,
}
