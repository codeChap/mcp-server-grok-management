//! Invoice list/detail and current-period preview.

use serde_json::Value;

use super::money::{money_cell, parse_cents};
use crate::util::{cents_signed_to_dollars, field, field_str};

pub fn fmt_invoice_preview(v: &Value) -> String {
    let cycle = field(v, &["billingCycle", "billing_cycle"]);
    let year = cycle
        .and_then(|c| field(c, &["year"]))
        .and_then(|y| y.as_i64())
        .unwrap_or(0);
    let month = cycle
        .and_then(|c| field(c, &["month"]))
        .and_then(|m| m.as_i64())
        .unwrap_or(0);
    let core = field(v, &["coreInvoice", "core_invoice"]).unwrap_or(v);
    let after = money_cell(field(core, &["amountAfterVat", "amount_after_vat"]));
    let before = money_cell(field(core, &["amountBeforeVat", "amount_before_vat"]));
    let vat = money_cell(field(core, &["vatCost", "vat_cost"]));
    let limit = money_cell(field(
        v,
        &["effectiveSpendingLimit", "effective_spending_limit"],
    ));
    let credits = money_cell(field(v, &["defaultCredits", "default_credits"]));
    let prepaid = money_cell(field(core, &["prepaidCredits", "prepaid_credits"]));
    let prepaid_used = money_cell(field(core, &["prepaidCreditsUsed", "prepaid_credits_used"]));

    let mut out = format!("Current postpaid period: {year:04}-{month:02}\n\n");
    out.push_str(&format!("  Amount after tax:   {after}\n"));
    out.push_str(&format!("  Amount before tax:  {before}\n"));
    out.push_str(&format!("  VAT:                {vat}\n"));
    out.push_str(&format!("  Effective spend limit: {limit}\n"));
    out.push_str(&format!("  Default credits:    {credits}\n"));
    out.push_str(&format!("  Prepaid credits:    {prepaid}\n"));
    out.push_str(&format!("  Prepaid used:       {prepaid_used}\n"));

    // Remaining headroom if we can parse
    if let (Some(spent), Some(lim)) = (
        parse_cents(field(core, &["amountAfterVat", "amount_after_vat"])),
        parse_cents(field(
            v,
            &["effectiveSpendingLimit", "effective_spending_limit"],
        )),
    ) {
        let remain = lim - spent;
        out.push_str(&format!(
            "  Headroom to soft limit: {}\n",
            cents_signed_to_dollars(remain)
        ));
    }

    let lines = field(core, &["lines"])
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !lines.is_empty() {
        // Aggregate by description
        let mut by_desc: Vec<(String, i64, i64)> = Vec::new(); // desc, amount_cents, units
        for line in &lines {
            let desc = field_str(line, &["description"]).unwrap_or_else(|| "(unknown)".into());
            let amt = parse_cents(field(line, &["amount"])).unwrap_or(0);
            let units = field_str(line, &["numUnits", "num_units"])
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);
            if let Some(row) = by_desc.iter_mut().find(|(d, _, _)| d == &desc) {
                row.1 += amt;
                row.2 += units;
            } else {
                by_desc.push((desc, amt, units));
            }
        }
        by_desc.sort_by(|a, b| b.1.cmp(&a.1));
        out.push_str(&format!(
            "\nLine items by product ({} raw lines → {} groups):\n",
            lines.len(),
            by_desc.len()
        ));
        for (desc, amt, _u) in by_desc.iter().take(25) {
            out.push_str(&format!(
                "  {:>10}  {}\n",
                cents_signed_to_dollars(*amt),
                desc
            ));
        }
        if by_desc.len() > 25 {
            out.push_str(&format!("  … +{} more groups\n", by_desc.len() - 25));
        }
        // Top unit-type breakdown for largest products
        out.push_str("\nTop raw lines (amount > $0):\n");
        let mut priced: Vec<&Value> = lines
            .iter()
            .filter(|l| parse_cents(field(l, &["amount"])).unwrap_or(0) > 0)
            .collect();
        priced.sort_by(|a, b| {
            parse_cents(field(b, &["amount"]))
                .unwrap_or(0)
                .cmp(&parse_cents(field(a, &["amount"])).unwrap_or(0))
        });
        for line in priced.iter().take(15) {
            let desc = field_str(line, &["description"]).unwrap_or_default();
            let unit = field_str(line, &["unitType", "unit_type"]).unwrap_or_default();
            let cluster = field_str(line, &["clusterName", "cluster_name"]).unwrap_or_default();
            let units = field_str(line, &["numUnits", "num_units"]).unwrap_or_default();
            let amt = money_cell(field(line, &["amount"]));
            out.push_str(&format!(
                "  {amt:>10}  {desc} | {unit} x{units} @ {cluster}\n"
            ));
        }
    } else {
        out.push_str("\n(no line items yet this period)\n");
    }
    out
}

pub fn fmt_invoices(v: &Value) -> String {
    let invs = field(v, &["invoices"])
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if invs.is_empty() {
        return "No invoices found.\n".into();
    }
    let mut out = format!("Invoices ({})\n\n", invs.len());
    out.push_str(&format_args_str(
        "  {:<12} {:<10} {:>10}  {:<7}  {}\n",
        &["created", "status", "total", "cycle", "invoice_id"],
    ));
    for inv in &invs {
        let created = field_str(inv, &["createTime", "create_time"])
            .map(|s| s.chars().take(10).collect::<String>())
            .unwrap_or_else(|| "-".into());
        let status = field_str(inv, &["invoiceStatus", "status"]).unwrap_or_else(|| "-".into());
        let total = money_cell(field(inv, &["total"]));
        let cycle = inv
            .get("monthly")
            .and_then(|m| m.get("billingCycle").or_else(|| m.get("billing_cycle")))
            .map(|c| {
                let y = c.get("year").and_then(|x| x.as_i64()).unwrap_or(0);
                let m = c.get("month").and_then(|x| x.as_i64()).unwrap_or(0);
                format!("{y:04}-{m:02}")
            })
            .unwrap_or_else(|| "-".into());
        let id = field_str(
            inv,
            &["invoiceId", "invoice_id", "invoiceNumber", "invoice_number"],
        )
        .unwrap_or_else(|| "-".into());
        // Truncate long ids for table
        let id_short = if id.len() > 24 {
            format!("{}…", &id[..20])
        } else {
            id
        };
        out.push_str(&format!(
            "  {created:<12} {status:<10} {total:>10}  {cycle:<7}  {id_short}\n"
        ));
    }
    out.push_str("\nUse get_invoice with invoice_id for line-item detail.\n");
    out
}

pub fn fmt_invoice_detail(inv: &Value) -> String {
    let id = field_str(inv, &["invoiceId", "invoice_id"]).unwrap_or_else(|| "-".into());
    let num = field_str(inv, &["invoiceNumber", "invoice_number"]).unwrap_or_default();
    let status = field_str(inv, &["invoiceStatus", "status"]).unwrap_or_else(|| "-".into());
    let created = field_str(inv, &["createTime", "create_time"]).unwrap_or_else(|| "-".into());
    let total = money_cell(field(inv, &["total"]));
    let sub = money_cell(field(inv, &["subtotal"]));
    let tax = money_cell(field(inv, &["tax"]));
    let mut out = String::from("Invoice detail\n");
    out.push_str(&format!("  ID:      {id}\n"));
    if !num.is_empty() {
        out.push_str(&format!("  Number:  {num}\n"));
    }
    out.push_str(&format!("  Status:  {status}\n"));
    out.push_str(&format!("  Created: {created}\n"));
    out.push_str(&format!("  Subtotal:{sub}\n"));
    out.push_str(&format!("  Tax:     {tax}\n"));
    out.push_str(&format!("  Total:   {total}\n"));
    if let Some(m) = inv.get("monthly") {
        if let Some(c) = m.get("billingCycle").or_else(|| m.get("billing_cycle")) {
            let y = c.get("year").and_then(|x| x.as_i64()).unwrap_or(0);
            let mo = c.get("month").and_then(|x| x.as_i64()).unwrap_or(0);
            out.push_str(&format!("  Cycle:   {y:04}-{mo:02}\n"));
        }
    }
    let lines = field(inv, &["lines"])
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !lines.is_empty() {
        out.push_str(&format!("\nLine items ({}):\n", lines.len()));
        for line in lines.iter().take(40) {
            let desc = field_str(line, &["description"]).unwrap_or_default();
            let unit = field_str(line, &["unitType", "unit_type"]).unwrap_or_default();
            let units = field_str(line, &["numUnits", "num_units"]).unwrap_or_default();
            let amt = money_cell(field(line, &["amount"]));
            out.push_str(&format!("  {amt:>10}  {desc} | {unit} x{units}\n"));
        }
        if lines.len() > 40 {
            out.push_str(&format!("  … +{} more lines\n", lines.len() - 40));
        }
    }
    out
}

fn format_args_str(template: &str, cols: &[&str]) -> String {
    // simple header helper
    let _ = template;
    format!(
        "  {:<12} {:<10} {:>10}  {:<7}  {}\n",
        cols[0], cols[1], cols[2], cols[3], cols[4]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn preview_formats_cycle() {
        let v = json!({
            "billingCycle": {"year": 2026, "month": 7},
            "effectiveSpendingLimit": "10000",
            "defaultCredits": "0",
            "coreInvoice": {
                "amountAfterVat": "1413",
                "amountBeforeVat": "1413",
                "vatCost": "0",
                "prepaidCredits": {"val": "0"},
                "prepaidCreditsUsed": {"val": "0"},
                "lines": [
                    {"description": "API grok-4.3", "amount": "500", "unitType": "tokens", "numUnits": "10", "clusterName": "us-west-2"}
                ]
            }
        });
        let s = fmt_invoice_preview(&v);
        assert!(s.contains("2026-07"));
        assert!(s.contains("$14.13"));
        assert!(s.contains("grok-4.3"));
    }
}
