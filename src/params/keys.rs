//! API key tool parameters.

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListApiKeysParams {
    #[schemars(description = "Max results per page (default 20).")]
    pub page_size: Option<u32>,
    #[schemars(description = "Pagination token from a previous list_api_keys call.")]
    pub pagination_token: Option<String>,
    #[schemars(
        description = "Optional ACL filters — only return keys that match these ACL strings."
    )]
    pub acl_filters: Option<Vec<String>>,
    #[schemars(description = "If true, only return non-expired / active keys (default false).")]
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateApiKeyParams {
    #[schemars(description = "Human-readable name for the API key.")]
    pub name: String,
    #[schemars(
        description = "ACL strings. Use [\"api-key:model:*\", \"api-key:endpoint:*\"] for full access."
    )]
    pub acls: Vec<String>,
    #[schemars(description = "Queries-per-second limit (optional).")]
    pub qps: Option<u32>,
    #[schemars(description = "Queries-per-minute limit (optional).")]
    pub qpm: Option<u32>,
    #[schemars(description = "Tokens-per-minute limit (optional; sent as string).")]
    pub tpm: Option<u64>,
    #[schemars(description = "Expiration time, RFC3339 (optional).")]
    pub expire_time: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateApiKeyParams {
    #[schemars(description = "API key ID (not the secret).")]
    pub api_key_id: String,
    pub name: Option<String>,
    pub acls: Option<Vec<String>>,
    pub qps: Option<u32>,
    pub qpm: Option<u32>,
    pub tpm: Option<u64>,
    pub disabled: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteApiKeyParams {
    pub api_key_id: String,
    #[schemars(description = "Must be exactly \"DELETE\".")]
    pub confirm: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RotateApiKeyParams {
    pub api_key_id: String,
    #[schemars(description = "Must be exactly \"ROTATE\".")]
    pub confirm: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ApiKeyIdParams {
    pub api_key_id: String,
}
