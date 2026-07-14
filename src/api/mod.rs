//! HTTP client for `https://management-api.x.ai`.
//!
//! Transport lives here; domain endpoints are `impl Api` blocks in submodules.

mod audit;
mod billing;
mod keys;

use anyhow::{Context, Result, anyhow, bail};
use reqwest::{Client, Method, RequestBuilder};
use serde_json::Value;
use std::time::Duration;
use tokio::sync::OnceCell;

use crate::config::Config;

/// Thin reqwest wrapper around the xAI Management API.
pub struct Api {
    client: Client,
    base_url: String,
    management_key: String,
    configured_team_id: Option<String>,
    discovered_team_id: OnceCell<String>,
}

impl Api {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .user_agent("mcp-server-grok-management")
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url: config.base_url().to_string(),
            management_key: config.management_key.clone(),
            configured_team_id: config.team_id.clone(),
            discovered_team_id: OnceCell::new(),
        }
    }

    fn req(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}{path}", self.base_url);
        self.client
            .request(method, url)
            .bearer_auth(&self.management_key)
    }

    async fn send(&self, builder: RequestBuilder) -> Result<Value> {
        let resp = builder
            .send()
            .await
            .context("request to xAI Management API failed")?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .with_context(|| format!("failed to read Management API body (HTTP {status})"))?;

        if text.trim().is_empty() {
            if status.is_success() {
                return Ok(Value::Null);
            }
            bail!("HTTP {status}: empty response body");
        }

        let body: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => {
                let snippet = truncate(&text, 500);
                if status.is_success() {
                    return Ok(Value::String(text));
                }
                return Err(api_err(status.as_u16(), &snippet));
            }
        };

        if !status.is_success() {
            return Err(api_err(status.as_u16(), &truncate(&text, 500)));
        }
        Ok(body)
    }

    pub async fn get(&self, path: &str, query: &[(String, String)]) -> Result<Value> {
        let mut builder = self.req(Method::GET, path);
        for (k, v) in query {
            builder = builder.query(&[(k.as_str(), v.as_str())]);
        }
        self.send(builder).await
    }

    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        self.send(self.req(Method::POST, path).json(body)).await
    }

    pub async fn put(&self, path: &str, body: &Value) -> Result<Value> {
        self.send(self.req(Method::PUT, path).json(body)).await
    }

    pub async fn delete(&self, path: &str) -> Result<Value> {
        self.send(self.req(Method::DELETE, path)).await
    }

    /// Validate the management key (meta, ACLs, IP ranges). No ACL required.
    pub async fn validate(&self) -> Result<Value> {
        self.get("/auth/management-keys/validation", &[]).await
    }

    /// Resolve team id from config or key validation (cached).
    pub async fn team_id(&self) -> Result<String> {
        if let Some(id) = &self.configured_team_id {
            let id = id.trim();
            if !id.is_empty() {
                return Ok(id.to_string());
            }
        }
        let id = self
            .discovered_team_id
            .get_or_try_init(|| async {
                let v = self.validate().await?;
                extract_team_id(&v).ok_or_else(|| {
                    let keys: Vec<String> = v
                        .as_object()
                        .map(|o| o.keys().cloned().collect())
                        .unwrap_or_default();
                    anyhow!(
                        "Could not auto-discover team_id from key validation; \
                         set team_id in config.toml. Response keys: {}",
                        keys.join(", ")
                    )
                })
            })
            .await?;
        Ok(id.clone())
    }
}

fn extract_team_id(v: &Value) -> Option<String> {
    for key in ["teamId", "scopeId", "team_id", "scope_id"] {
        if let Some(s) = v.get(key).and_then(Value::as_str) {
            let s = s.trim();
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn api_err(status: u16, snippet: &str) -> anyhow::Error {
    if status == 403 {
        anyhow!(
            "Permission denied (403) — your management key lacks the required ACL for this endpoint: {snippet}"
        )
    } else {
        anyhow!("HTTP {status}: {snippet}")
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max).collect();
        format!("{t}…")
    }
}
