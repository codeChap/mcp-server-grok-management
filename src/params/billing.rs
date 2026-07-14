//! Billing tool parameters.

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListInvoicesParams {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub since_year: Option<i32>,
    pub since_month: Option<i32>,
    pub invoice_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetInvoiceParams {
    pub invoice_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetSpendingLimitParams {
    #[schemars(description = "Soft limit in USD cents (e.g. 10000 = $100).")]
    pub limit_cents: u64,
    #[schemars(description = "Must be exactly \"YES-WRITE\".")]
    pub confirm: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TopUpParams {
    #[schemars(description = "Amount in USD cents. CHARGES the default payment method.")]
    pub amount_cents: u64,
    #[schemars(description = "Must be exactly \"TOP-UP\".")]
    pub confirm: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetDefaultPaymentMethodParams {
    pub payment_method_id: String,
    #[schemars(description = "Must be exactly \"YES-WRITE\".")]
    pub confirm: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetBillingInfoParams {
    pub name: String,
    pub email: String,
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub tax_id_type: Option<String>,
    pub tax_number: Option<String>,
    #[schemars(description = "Must be exactly \"YES-WRITE\".")]
    pub confirm: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetUsageParams {
    pub time_from: String,
    pub time_to: String,
    pub time_unit: Option<String>,
    pub timezone: Option<String>,
    pub group_by: Option<Vec<String>>,
    pub metric: Option<String>,
}
