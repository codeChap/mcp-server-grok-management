//! API key, model, and endpoint Management API methods.

use anyhow::Result;
use serde_json::{Value, json};

use super::Api;

impl Api {
    pub async fn list_api_keys(
        &self,
        team_id: &str,
        query: &[(String, String)],
    ) -> Result<Value> {
        self.get(&format!("/auth/teams/{team_id}/api-keys"), query)
            .await
    }

    pub async fn create_api_key(&self, team_id: &str, body: &Value) -> Result<Value> {
        self.post(&format!("/auth/teams/{team_id}/api-keys"), body)
            .await
    }

    pub async fn update_api_key(&self, api_key_id: &str, body: &Value) -> Result<Value> {
        self.put(&format!("/auth/api-keys/{api_key_id}"), body)
            .await
    }

    pub async fn delete_api_key(&self, api_key_id: &str) -> Result<Value> {
        self.delete(&format!("/auth/api-keys/{api_key_id}")).await
    }

    pub async fn rotate_api_key(&self, api_key_id: &str) -> Result<Value> {
        self.post(&format!("/auth/api-keys/{api_key_id}/rotate"), &json!({}))
            .await
    }

    pub async fn check_propagation(&self, api_key_id: &str) -> Result<Value> {
        self.get(&format!("/auth/api-keys/{api_key_id}/propagation"), &[])
            .await
    }

    pub async fn list_models(&self, team_id: &str) -> Result<Value> {
        self.get(&format!("/auth/teams/{team_id}/models"), &[])
            .await
    }

    pub async fn list_endpoints(&self, team_id: &str) -> Result<Value> {
        self.get(&format!("/auth/teams/{team_id}/endpoints"), &[])
            .await
    }
}
