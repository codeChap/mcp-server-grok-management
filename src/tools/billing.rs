//! Billing tools.

use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::*,
    tool, tool_router,
};
use serde_json::{Value, json};

use crate::format::billing as fmt;
use crate::helpers::{err, err_text, map_fmt, ok, require_confirm, team, with_team, with_team_fmt};
use crate::params::*;
use crate::tools::GrokManagementServer;
use crate::util::{
    array_field, cents_to_dollars, field, field_str, map_time_unit, normalize_usage_timestamp,
    pretty,
};

#[tool_router(router = router_billing, vis = "pub")]
impl GrokManagementServer {
    #[tool(
        description = "One-shot billing dashboard: period spend, headroom, limits, prepaid, payment methods, contact."
    )]
    async fn billing_overview(
        &self,
        Parameters(_p): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        let team_id = match team(&self.api).await {
            Ok(t) => t,
            Err(e) => return Ok(e),
        };
        let (preview, limits, prepaid, methods, info) = tokio::join!(
            self.api.invoice_preview(&team_id),
            self.api.spending_limits(&team_id),
            self.api.prepaid_balance(&team_id),
            self.api.payment_methods(&team_id),
            self.api.billing_info(&team_id),
        );

        let mut out = format!("=== Billing overview (team {team_id}) ===\n\n");
        section(&mut out, "Current period", preview, fmt::fmt_invoice_preview);
        section(&mut out, "Spending limits", limits, fmt::fmt_spending_limits);
        match prepaid {
            Ok(v) => {
                out.push_str("--- Prepaid ---\n");
                if let Some(t) = fmt::parse_cents(field(&v, &["total"])) {
                    let available = if t <= 0 { -t } else { t };
                    out.push_str(&format!(
                        "  Available credit (approx): {}\n  Ledger raw cents: {t}\n\n",
                        cents_to_dollars(available as u64)
                    ));
                } else {
                    out.push_str(&fmt::fmt_prepaid(&v));
                    out.push('\n');
                }
            }
            Err(e) => out.push_str(&format!("--- Prepaid ---\nERROR: {e}\n\n")),
        }
        section(&mut out, "Payment methods", methods, fmt::fmt_payment_methods);
        section(&mut out, "Billing contact", info, fmt::fmt_billing_info);
        Ok(ok(out))
    }

    #[tool(description = "Get team billing contact name, email, address, tax ids.")]
    async fn get_billing_info(
        &self,
        Parameters(_p): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        let api = self.api.clone();
        Ok(with_team_fmt(
            &self.api,
            move |tid| {
                let api = api.clone();
                async move { api.billing_info(&tid).await }
            },
            |_t, v| fmt::fmt_billing_info(v),
        )
        .await)
    }

    #[tool(
        description = "Update team billing contact/address. Requires confirm=\"YES-WRITE\"."
    )]
    async fn set_billing_info(
        &self,
        Parameters(p): Parameters<SetBillingInfoParams>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = require_confirm(
            &p.confirm,
            "YES-WRITE",
            "Billing info NOT changed. Pass confirm = \"YES-WRITE\".",
        ) {
            return Ok(e);
        }
        let body = json!({
            "billingInfo": {
                "name": p.name,
                "email": p.email,
                "address": {
                    "line1": p.line1.unwrap_or_default(),
                    "line2": p.line2.unwrap_or_default(),
                    "city": p.city.unwrap_or_default(),
                    "state": p.state.unwrap_or_default(),
                    "postalCode": p.postal_code.unwrap_or_default(),
                    "country": p.country.unwrap_or_default(),
                },
                "taxIdType": p.tax_id_type.unwrap_or_default(),
                "taxNumber": p.tax_number.unwrap_or_default(),
            }
        });
        let team_id = match team(&self.api).await {
            Ok(t) => t,
            Err(e) => return Ok(e),
        };
        if let Err(e) = self.api.set_billing_info(&team_id, &body).await {
            return Ok(err(e));
        }
        Ok(map_fmt(self.api.billing_info(&team_id).await, |v| {
            format!("Billing info updated.\n\n{}", fmt::fmt_billing_info(v))
        }))
    }

    #[tool(
        description = "List invoices. Optional filters: year+month cycle, since_year/since_month, invoice_ids."
    )]
    async fn list_invoices(
        &self,
        Parameters(p): Parameters<ListInvoicesParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut query: Vec<(String, String)> = Vec::new();
        if let Some(y) = p.year {
            query.push(("billingCycle.year".into(), y.to_string()));
        }
        if let Some(m) = p.month {
            query.push(("billingCycle.month".into(), m.to_string()));
        }
        if let Some(y) = p.since_year {
            query.push(("since.year".into(), y.to_string()));
        }
        if let Some(m) = p.since_month {
            query.push(("since.month".into(), m.to_string()));
        }
        if let Some(ids) = &p.invoice_ids {
            for id in ids {
                query.push(("invoiceIds.invoiceIds".into(), id.clone()));
            }
        }
        let api = self.api.clone();
        Ok(with_team_fmt(
            &self.api,
            move |tid| {
                let api = api.clone();
                async move { api.list_invoices(&tid, &query).await }
            },
            |_t, v| fmt::fmt_invoices(v),
        )
        .await)
    }

    #[tool(description = "Get one invoice with line items by invoice_id.")]
    async fn get_invoice(
        &self,
        Parameters(p): Parameters<GetInvoiceParams>,
    ) -> Result<CallToolResult, McpError> {
        let id = p.invoice_id.trim().to_string();
        let query = vec![("invoiceIds.invoiceIds".into(), id.clone())];
        let api = self.api.clone();
        Ok(with_team(
            &self.api,
            move |tid| {
                let api = api.clone();
                async move { api.list_invoices(&tid, &query).await }
            },
            move |_tid, v| {
                let invs = array_field(&v, &["invoices"]);
                if invs.is_empty() {
                    return format!("No invoice found for id {id}");
                }
                let inv = invs
                    .iter()
                    .find(|i| {
                        field_str(i, &["invoiceId", "invoice_id"]).as_deref() == Some(id.as_str())
                    })
                    .unwrap_or(&invs[0]);
                fmt::fmt_invoice_detail(inv)
            },
        )
        .await)
    }

    #[tool(
        description = "List payment methods (card/Link/ACH). Add cards in console; set_default_payment_method chooses default for top-ups."
    )]
    async fn list_payment_methods(
        &self,
        Parameters(_p): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        let api = self.api.clone();
        Ok(with_team_fmt(
            &self.api,
            move |tid| {
                let api = api.clone();
                async move { api.payment_methods(&tid).await }
            },
            |_t, v| fmt::fmt_payment_methods(v),
        )
        .await)
    }

    #[tool(
        description = "Set default payment method for charges/top-ups. Requires confirm=\"YES-WRITE\"."
    )]
    async fn set_default_payment_method(
        &self,
        Parameters(p): Parameters<SetDefaultPaymentMethodParams>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = require_confirm(
            &p.confirm,
            "YES-WRITE",
            "Default payment method NOT changed. Pass confirm = \"YES-WRITE\".",
        ) {
            return Ok(e);
        }
        let pm = p.payment_method_id.trim().to_string();
        let api = self.api.clone();
        Ok(with_team(
            &self.api,
            move |tid| {
                let api = api.clone();
                let pm = pm.clone();
                async move { api.set_default_payment_method(&tid, &pm).await }
            },
            move |_t, _v| format!("Default payment method set to {}.", p.payment_method_id.trim()),
        )
        .await)
    }

    #[tool(
        description = "Current postpaid period preview: spend, VAT, headroom, product breakdown."
    )]
    async fn get_invoice_preview(
        &self,
        Parameters(_p): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        let api = self.api.clone();
        Ok(with_team_fmt(
            &self.api,
            move |tid| {
                let api = api.clone();
                async move { api.invoice_preview(&tid).await }
            },
            |_t, v| fmt::fmt_invoice_preview(v),
        )
        .await)
    }

    #[tool(description = "Get postpaid soft/hard spending limits in USD.")]
    async fn get_spending_limits(
        &self,
        Parameters(_p): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        let api = self.api.clone();
        Ok(with_team_fmt(
            &self.api,
            move |tid| {
                let api = api.clone();
                async move { api.spending_limits(&tid).await }
            },
            |_t, v| fmt::fmt_spending_limits(v),
        )
        .await)
    }

    #[tool(
        description = "Set postpaid soft spending limit (USD cents). Requires confirm=\"YES-WRITE\". 0 = prepaid-only postpaid cap."
    )]
    async fn set_spending_limit(
        &self,
        Parameters(p): Parameters<SetSpendingLimitParams>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = require_confirm(
            &p.confirm,
            "YES-WRITE",
            "Spending limit NOT changed. Pass confirm = \"YES-WRITE\" to set the soft limit.",
        ) {
            return Ok(e);
        }
        let cents = p.limit_cents;
        let api = self.api.clone();
        Ok(with_team(
            &self.api,
            move |tid| {
                let api = api.clone();
                async move { api.set_spending_limit(&tid, cents).await }
            },
            move |_t, v| {
                let applied = fmt::money_cell(field(&v, &["thisBpSoftSpendingLimit"]));
                format!(
                    "Soft spending limit set to {} ({} cents).\nAPI response applied: {applied}\n\n\
                     Re-check with get_spending_limits / billing_overview.",
                    cents_to_dollars(cents),
                    cents
                )
            },
        )
        .await)
    }

    #[tool(description = "Prepaid credit balance and recent purchase/spend/refund ledger.")]
    async fn get_prepaid_balance(
        &self,
        Parameters(_p): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        let api = self.api.clone();
        Ok(with_team_fmt(
            &self.api,
            move |tid| {
                let api = api.clone();
                async move { api.prepaid_balance(&tid).await }
            },
            |_t, v| fmt::fmt_prepaid(v),
        )
        .await)
    }

    #[tool(
        description = "Top up prepaid credits (CHARGES card). amount_cents e.g. 2500=$25. Requires confirm=\"TOP-UP\"."
    )]
    async fn top_up(
        &self,
        Parameters(p): Parameters<TopUpParams>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = require_confirm(
            &p.confirm,
            "TOP-UP",
            "Top-up NOT executed. This CHARGES the team payment card. \
             Pass confirm = \"TOP-UP\" with amount_cents (e.g. 2500 = $25.00).",
        ) {
            return Ok(e);
        }
        if p.amount_cents == 0 {
            return Ok(err_text("amount_cents must be greater than 0."));
        }
        let cents = p.amount_cents;
        let api = self.api.clone();
        Ok(with_team(
            &self.api,
            move |tid| {
                let api = api.clone();
                async move { api.top_up(&tid, cents).await }
            },
            move |_t, v| {
                let status = v
                    .get("change")
                    .and_then(|c| field_str(c, &["topupStatus", "topup_status"]))
                    .unwrap_or_else(|| "-".into());
                let inv = v
                    .get("change")
                    .and_then(|c| field_str(c, &["invoiceNumber", "invoice_number", "invoiceId"]))
                    .unwrap_or_default();
                format!(
                    "Top-up of {} ({} cents) submitted.\nStatus: {status}\nInvoice: {inv}\n\n{}",
                    cents_to_dollars(cents),
                    cents,
                    pretty(&v)
                )
            },
        )
        .await)
    }

    #[tool(
        description = "Usage/cost analytics. Defaults: metric=usd SUM, group_by=[description], timezone=Etc/GMT."
    )]
    async fn get_usage(
        &self,
        Parameters(p): Parameters<GetUsageParams>,
    ) -> Result<CallToolResult, McpError> {
        let unit = match map_time_unit(p.time_unit.as_deref()) {
            Ok(u) => u,
            Err(msg) => return Ok(err_text(msg)),
        };
        let start = normalize_usage_timestamp(&p.time_from);
        let end = normalize_usage_timestamp(&p.time_to);
        let tz = p
            .timezone
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("Etc/GMT");
        let metric = p
            .metric
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("usd");
        let group_by = p
            .group_by
            .clone()
            .filter(|g| !g.is_empty())
            .unwrap_or_else(|| vec!["description".into()]);
        let body = json!({
            "analyticsRequest": {
                "timeRange": {
                    "startTime": start,
                    "endTime": end,
                    "timezone": tz
                },
                "timeUnit": unit,
                "values": [{ "name": metric, "aggregation": "AGGREGATION_SUM" }],
                "groupBy": group_by,
                "filters": []
            }
        });
        let api = self.api.clone();
        let start_c = start.clone();
        let end_c = end.clone();
        Ok(with_team_fmt(
            &self.api,
            move |tid| {
                let api = api.clone();
                async move { api.usage(&tid, &body).await }
            },
            move |_t, v| fmt::fmt_usage(v, &start_c, &end_c, unit),
        )
        .await)
    }
}

fn section(
    out: &mut String,
    title: &str,
    result: anyhow::Result<Value>,
    fmt_fn: impl FnOnce(&Value) -> String,
) {
    match result {
        Ok(v) => {
            out.push_str(&format!("--- {title} ---\n"));
            out.push_str(&fmt_fn(&v));
            out.push('\n');
        }
        Err(e) => out.push_str(&format!("--- {title} ---\nERROR: {e}\n\n")),
    }
}
