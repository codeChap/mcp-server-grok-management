//! Billing Management API methods.

use anyhow::Result;
use serde_json::{Value, json};

use super::Api;

impl Api {
    pub async fn billing_info(&self, team_id: &str) -> Result<Value> {
        self.get(&format!("/v1/billing/teams/{team_id}/billing-info"), &[])
            .await
    }

    pub async fn set_billing_info(&self, team_id: &str, body: &Value) -> Result<Value> {
        self.post(&format!("/v1/billing/teams/{team_id}/billing-info"), body)
            .await
    }

    pub async fn list_invoices(
        &self,
        team_id: &str,
        query: &[(String, String)],
    ) -> Result<Value> {
        self.get(&format!("/v1/billing/teams/{team_id}/invoices"), query)
            .await
    }

    pub async fn payment_methods(&self, team_id: &str) -> Result<Value> {
        self.get(
            &format!("/v1/billing/teams/{team_id}/payment-method"),
            &[],
        )
        .await
    }

    pub async fn set_default_payment_method(
        &self,
        team_id: &str,
        payment_method_id: &str,
    ) -> Result<Value> {
        self.post(
            &format!("/v1/billing/teams/{team_id}/payment-method/default"),
            &json!({ "paymentMethodId": payment_method_id }),
        )
        .await
    }

    pub async fn invoice_preview(&self, team_id: &str) -> Result<Value> {
        self.get(
            &format!("/v1/billing/teams/{team_id}/postpaid/invoice/preview"),
            &[],
        )
        .await
    }

    pub async fn spending_limits(&self, team_id: &str) -> Result<Value> {
        self.get(
            &format!("/v1/billing/teams/{team_id}/postpaid/spending-limits"),
            &[],
        )
        .await
    }

    pub async fn set_spending_limit(&self, team_id: &str, limit_cents: u64) -> Result<Value> {
        self.post(
            &format!("/v1/billing/teams/{team_id}/postpaid/spending-limits"),
            &json!({
                "desiredSoftSpendingLimit": { "val": limit_cents.to_string() }
            }),
        )
        .await
    }

    pub async fn prepaid_balance(&self, team_id: &str) -> Result<Value> {
        self.get(
            &format!("/v1/billing/teams/{team_id}/prepaid/balance"),
            &[],
        )
        .await
    }

    pub async fn top_up(&self, team_id: &str, amount_cents: u64) -> Result<Value> {
        self.post(
            &format!("/v1/billing/teams/{team_id}/prepaid/top-up"),
            &json!({ "amount": { "val": amount_cents.to_string() } }),
        )
        .await
    }

    pub async fn usage(&self, team_id: &str, body: &Value) -> Result<Value> {
        self.post(&format!("/v1/billing/teams/{team_id}/usage"), body)
            .await
    }
}
