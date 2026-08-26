//! API key tools.

use rmcp::{
    ErrorData as McpError, handler::server::wrapper::Parameters, model::*, tool, tool_router,
};
use serde_json::json;

use crate::format::keys as fmt;
use crate::helpers::{err_text, map_fmt, require_confirm, with_team, with_team_fmt};
use crate::params::*;
use crate::tools::GrokManagementServer;

#[tool_router(router = router_keys, vis = "pub")]
impl GrokManagementServer {
    #[tool(
        description = "Validate the management key; show name, team id, ACLs (BillingRead/Write), IP ranges."
    )]
    async fn validate_key(
        &self,
        Parameters(_p): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        Ok(map_fmt(self.api.validate().await, fmt::fmt_validate_key))
    }

    #[tool(
        description = "List inference API keys. Optional page_size, pagination_token, acl_filters, active_only."
    )]
    async fn list_api_keys(
        &self,
        Parameters(p): Parameters<ListApiKeysParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut query = vec![("pageSize".into(), p.page_size.unwrap_or(20).to_string())];
        if let Some(tok) = p.pagination_token.filter(|s| !s.is_empty()) {
            query.push(("paginationToken".into(), tok));
        }
        if let Some(true) = p.active_only {
            query.push(("activeOnly".into(), "true".into()));
        }
        if let Some(filters) = &p.acl_filters {
            for f in filters {
                query.push(("aclFilters".into(), f.clone()));
            }
        }
        let api = self.api.clone();
        Ok(with_team_fmt(
            &self.api,
            move |tid| {
                let api = api.clone();
                async move { api.list_api_keys(&tid, &query).await }
            },
            fmt::fmt_list_api_keys,
        )
        .await)
    }

    #[tool(
        description = "Create an inference API key. Secret shown ONCE. Full access: acls=[\"api-key:model:*\",\"api-key:endpoint:*\"]."
    )]
    async fn create_api_key(
        &self,
        Parameters(p): Parameters<CreateApiKeyParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut body = json!({ "name": p.name, "acls": p.acls });
        if let Some(q) = p.qps {
            body["qps"] = json!(q);
        }
        if let Some(q) = p.qpm {
            body["qpm"] = json!(q);
        }
        if let Some(t) = p.tpm {
            body["tpm"] = json!(t.to_string());
        }
        if let Some(e) = p.expire_time {
            body["expireTime"] = json!(e);
        }
        let empty_acls = p.acls.is_empty();
        let api = self.api.clone();
        Ok(with_team(
            &self.api,
            move |tid| {
                let api = api.clone();
                async move { api.create_api_key(&tid, &body).await }
            },
            move |_tid, v| {
                let mut out = String::new();
                if empty_acls {
                    out.push_str(
                        "WARNING: empty ACL list — this key has NO access until update_api_key.\n\n",
                    );
                }
                out.push_str(&fmt::format_new_secret(&v));
                out.push('\n');
                out.push_str(&fmt::fmt_api_key_row(&v));
                out.push('\n');
                out
            },
        )
        .await)
    }

    #[tool(description = "Selectively update an API key (name, acls, qps, qpm, tpm, disabled).")]
    async fn update_api_key(
        &self,
        Parameters(p): Parameters<UpdateApiKeyParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut api_key = json!({});
        let mut mask: Vec<&str> = Vec::new();
        if let Some(n) = &p.name {
            api_key["name"] = json!(n);
            mask.push("name");
        }
        if let Some(a) = &p.acls {
            api_key["aclStrings"] = json!(a);
            mask.push("aclStrings");
        }
        if let Some(q) = p.qps {
            api_key["qps"] = json!(q);
            mask.push("qps");
        }
        if let Some(q) = p.qpm {
            api_key["qpm"] = json!(q);
            mask.push("qpm");
        }
        if let Some(t) = p.tpm {
            api_key["tpm"] = json!(t.to_string());
            mask.push("tpm");
        }
        if let Some(d) = p.disabled {
            api_key["disabled"] = json!(d);
            mask.push("disabled");
        }
        if mask.is_empty() {
            return Ok(err_text(
                "Nothing to update: provide at least one of name, acls, qps, qpm, tpm, disabled.",
            ));
        }
        let body = json!({ "apiKey": api_key, "fieldMask": mask.join(",") });
        let id = p.api_key_id.trim().to_string();
        let mask_s = mask.join(", ");
        Ok(map_fmt(self.api.update_api_key(&id, &body).await, |v| {
            format!(
                "Updated API key {id} (fields: {mask_s}):\n\n{}",
                fmt::fmt_api_key_row(v)
            )
        }))
    }

    #[tool(description = "Permanently delete an API key. Requires confirm=\"DELETE\".")]
    async fn delete_api_key(
        &self,
        Parameters(p): Parameters<DeleteApiKeyParams>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = require_confirm(
            &p.confirm,
            "DELETE",
            "Deletion NOT executed. Pass confirm = \"DELETE\" to permanently delete this API key.",
        ) {
            return Ok(e);
        }
        let id = p.api_key_id.trim().to_string();
        Ok(map_fmt(self.api.delete_api_key(&id).await, |_| {
            format!("Deleted API key {id}.")
        }))
    }

    #[tool(
        description = "Rotate an API key secret. Requires confirm=\"ROTATE\". New secret shown ONCE."
    )]
    async fn rotate_api_key(
        &self,
        Parameters(p): Parameters<RotateApiKeyParams>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = require_confirm(
            &p.confirm,
            "ROTATE",
            "Rotation NOT executed. Pass confirm = \"ROTATE\". \
             Rotation invalidates the current secret (new secret returned once).",
        ) {
            return Ok(e);
        }
        let id = p.api_key_id.trim().to_string();
        Ok(map_fmt(self.api.rotate_api_key(&id).await, |v| {
            let mut out = format!("Rotated API key {id}.\n\n");
            out.push_str(&fmt::format_new_secret(v));
            out.push('\n');
            out.push_str(&fmt::fmt_api_key_row(v));
            out.push('\n');
            out
        }))
    }

    #[tool(description = "Check API key propagation across inference clusters.")]
    async fn check_propagation(
        &self,
        Parameters(p): Parameters<ApiKeyIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let id = p.api_key_id.trim().to_string();
        Ok(map_fmt(self.api.check_propagation(&id).await, |v| {
            fmt::fmt_propagation(&id, v)
        }))
    }

    #[tool(description = "List models for api-key:model:<name> ACLs.")]
    async fn list_models(
        &self,
        Parameters(_p): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        let api = self.api.clone();
        Ok(with_team_fmt(
            &self.api,
            move |tid| {
                let api = api.clone();
                async move { api.list_models(&tid).await }
            },
            fmt::fmt_list_models,
        )
        .await)
    }

    #[tool(description = "List endpoint ACLs for api-key:endpoint:<name>.")]
    async fn list_endpoints(
        &self,
        Parameters(_p): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        let api = self.api.clone();
        Ok(with_team_fmt(
            &self.api,
            move |tid| {
                let api = api.clone();
                async move { api.list_endpoints(&tid).await }
            },
            fmt::fmt_list_endpoints,
        )
        .await)
    }
}
