//! Prepaid balance ledger formatting.

use serde_json::Value;

use super::money::parse_cents;
use crate::util::{cents_signed_to_dollars, cents_to_dollars, field, field_str};

pub fn fmt_prepaid(v: &Value) -> String {
    let total = parse_cents(field(v, &["total"]));
    // API: purchases often negative val meaning credit added; total is signed ledger
    let mut out = String::from("Prepaid credit balance\n\n");
    match total {
        Some(t) => {
            // Display absolute credit available: docs say PURCHASE amounts are negative
            // total val of 0 = empty. Negative total often means credit remaining.
            let available = if t <= 0 { -t } else { t };
            out.push_str(&format!(
                "  Ledger total (raw cents): {t}\n  Available credit (approx): {}\n",
                cents_to_dollars(available as u64)
            ));
        }
        None => out.push_str("  Total: (unknown)\n"),
    }
    let changes = field(v, &["changes"])
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    out.push_str(&format!("\nRecent balance changes ({}, showing up to 20):\n", changes.len()));
    // Show newest first if API returns oldest-first
    let mut rows = changes;
    if rows.len() > 1 {
        // Prefer createTime order descending
        rows.sort_by(|a, b| {
            let ta = field_str(a, &["createTime", "create_ts", "createTs"]).unwrap_or_default();
            let tb = field_str(b, &["createTime", "create_ts", "createTs"]).unwrap_or_default();
            tb.cmp(&ta)
        });
    }
    for ch in rows.iter().take(20) {
        let origin = field_str(ch, &["changeOrigin", "change_origin"]).unwrap_or_else(|| "-".into());
        let status = field_str(ch, &["topupStatus", "topup_status"]).unwrap_or_default();
        let amt = parse_cents(field(ch, &["amount"])).unwrap_or(0);
        // PURCHASE negative => credit +$X
        let amt_label = match origin.as_str() {
            "PURCHASE" | "AUTO_PURCHASE" | "REFUND" if amt < 0 => {
                format!("credit {}", cents_to_dollars((-amt) as u64))
            }
            "SPEND" if amt > 0 => format!("spend  {}", cents_to_dollars(amt as u64)),
            _ => cents_signed_to_dollars(amt),
        };
        let when = field_str(ch, &["createTime", "createTs", "create_ts"]).unwrap_or_else(|| {
            let y = ch.get("spendBpKeyYear").and_then(|x| x.as_i64());
            let m = ch.get("spendBpKeyMonth").and_then(|x| x.as_i64());
            match (y, m) {
                (Some(y), Some(m)) => format!("{y:04}-{m:02}"),
                _ => "-".into(),
            }
        });
        let inv = field_str(ch, &["invoice_number", "invoiceNumber", "invoiceId", "invoice_id"])
            .unwrap_or_default();
        let inv_s = if inv.is_empty() {
            String::new()
        } else if inv.len() > 16 {
            format!(" inv={}", &inv[..12])
        } else {
            format!(" inv={inv}")
        };
        let st = if status.is_empty() {
            String::new()
        } else {
            format!(" [{status}]")
        };
        out.push_str(&format!(
            "  {when:<24} {origin:<14} {amt_label:<16}{st}{inv_s}\n"
        ));
    }
    out.push_str("\nTop up with top_up (confirm=\"TOP-UP\", amount_cents=…).\n");
    out
}

