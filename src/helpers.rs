//! Shared MCP tool plumbing — results, confirm gates, team resolution.

use anyhow::Result;
use rmcp::model::{CallToolResult, Content};
use serde_json::Value;
use std::future::Future;

use crate::api::Api;

pub fn ok(text: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![Content::text(text.into())])
}

pub fn err_text(text: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![Content::text(text.into())])
}

pub fn err(e: impl std::fmt::Display) -> CallToolResult {
    CallToolResult::error(vec![Content::text(e.to_string())])
}

/// Map an API `Result<Value>` through a formatter.
pub fn map_fmt(result: Result<Value>, fmt: impl FnOnce(&Value) -> String) -> CallToolResult {
    match result {
        Ok(v) => ok(fmt(&v)),
        Err(e) => err(e),
    }
}

/// Refuse unless `got == expected`.
pub fn require_confirm(
    got: &str,
    expected: &str,
    refuse_message: &str,
) -> Result<(), CallToolResult> {
    if got == expected {
        Ok(())
    } else {
        Err(err_text(refuse_message.to_string()))
    }
}

/// Resolve team id or return an error tool result.
pub async fn team(api: &Api) -> Result<String, CallToolResult> {
    api.team_id().await.map_err(err)
}

/// Run `f(team_id)` and format with `fmt(team_id, &value)`.
pub async fn with_team_fmt<F, Fut>(
    api: &Api,
    f: F,
    fmt: impl FnOnce(&str, &Value) -> String,
) -> CallToolResult
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Result<Value>>,
{
    let team_id = match team(api).await {
        Ok(t) => t,
        Err(e) => return e,
    };
    match f(team_id.clone()).await {
        Ok(v) => ok(fmt(&team_id, &v)),
        Err(e) => err(e),
    }
}

/// Run `f(team_id)` and map success via `on_ok(team_id, value)`.
pub async fn with_team<F, Fut>(
    api: &Api,
    f: F,
    on_ok: impl FnOnce(&str, Value) -> String,
) -> CallToolResult
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Result<Value>>,
{
    let team_id = match team(api).await {
        Ok(t) => t,
        Err(e) => return e,
    };
    match f(team_id.clone()).await {
        Ok(v) => ok(on_ok(&team_id, v)),
        Err(e) => err(e),
    }
}
