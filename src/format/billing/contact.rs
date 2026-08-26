//! Billing contact, payment methods, spending limits.

use serde_json::Value;

use super::money::money_cell;
use crate::util::{field, field_str};

pub fn fmt_billing_info(v: &Value) -> String {
    let info = field(v, &["billingInfo", "billing_info"]).unwrap_or(v);
    let name = field_str(info, &["name"]).unwrap_or_else(|| "(no name)".into());
    let email = field_str(info, &["email"]).unwrap_or_else(|| "-".into());
    let tax_type = field_str(info, &["taxIdType", "tax_id_type"]).unwrap_or_default();
    let tax_num = field_str(info, &["taxNumber", "tax_number"]).unwrap_or_default();
    let addr = field(info, &["address"]);
    let mut out = String::from("Billing contact\n");
    out.push_str(&format!("  Name:  {name}\n"));
    out.push_str(&format!("  Email: {email}\n"));
    if let Some(a) = addr {
        let line1 = field_str(a, &["line1"]).unwrap_or_default();
        let line2 = field_str(a, &["line2"]).unwrap_or_default();
        let city = field_str(a, &["city"]).unwrap_or_default();
        let state = field_str(a, &["state"]).unwrap_or_default();
        let postal = field_str(a, &["postalCode", "postal_code"]).unwrap_or_default();
        let country = field_str(a, &["country"]).unwrap_or_default();
        out.push_str("  Address:\n");
        if !line1.is_empty() {
            out.push_str(&format!("    {line1}\n"));
        }
        if !line2.is_empty() {
            out.push_str(&format!("    {line2}\n"));
        }
        let city_line = [city, state, postal]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ");
        if !city_line.is_empty() {
            out.push_str(&format!("    {city_line}\n"));
        }
        if !country.is_empty() {
            out.push_str(&format!("    {country}\n"));
        }
    }
    if !tax_type.is_empty() || !tax_num.is_empty() {
        out.push_str(&format!("  Tax: {tax_type} {tax_num}\n"));
    }
    out
}

pub fn fmt_payment_methods(v: &Value) -> String {
    let methods = field(v, &["paymentMethods", "payment_methods"])
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if methods.is_empty() {
        return "No payment methods on file.\n(Add cards at console.x.ai — the Management API cannot create payment methods.)\n".into();
    }
    let mut out = format!("Payment methods ({})\n\n", methods.len());
    for (i, m) in methods.iter().enumerate() {
        let id =
            field_str(m, &["paymentMethodId", "payment_method_id"]).unwrap_or_else(|| "-".into());
        let ptype = field_str(m, &["paymentType", "payment_type"]).unwrap_or_else(|| "-".into());
        let added = field_str(m, &["addedTs", "added_ts"]).unwrap_or_default();
        let bi = field(m, &["billingInfo", "billing_info"]);
        let bname = bi.and_then(|b| field_str(b, &["name"])).unwrap_or_default();
        let bemail = bi
            .and_then(|b| field_str(b, &["email"]))
            .unwrap_or_default();
        let detail = if let Some(card) = field(m, &["cardDetails", "card_details"]) {
            let brand = field_str(card, &["brand"]).unwrap_or_default();
            let last4 = field_str(card, &["last4"]).unwrap_or_default();
            let exp_m = field_str(card, &["expMonth", "exp_month"]).unwrap_or_default();
            let exp_y = field_str(card, &["expYear", "exp_year"]).unwrap_or_default();
            format!("{brand} ****{last4} exp {exp_m}/{exp_y}")
        } else if let Some(link) = field(m, &["linkDetails", "link_details"]) {
            let email = field_str(link, &["email"]).unwrap_or_default();
            format!("Stripe Link ({email})")
        } else if let Some(bank) = field(m, &["usBankAccountDetails", "us_bank_account_details"]) {
            let bank_name = field_str(bank, &["bankName", "bank_name"]).unwrap_or_default();
            let last4 = field_str(bank, &["last4"]).unwrap_or_default();
            format!("ACH {bank_name} ****{last4}")
        } else {
            ptype.clone()
        };
        out.push_str(&format!("{}. {id}\n", i + 1));
        out.push_str(&format!("   type:    {ptype}\n"));
        out.push_str(&format!("   detail:  {detail}\n"));
        if !bname.is_empty() {
            out.push_str(&format!("   name:    {bname}\n"));
        }
        if !bemail.is_empty() {
            out.push_str(&format!("   email:   {bemail}\n"));
        }
        if !added.is_empty() {
            out.push_str(&format!("   added:   {added}\n"));
        }
        out.push('\n');
    }
    if let Some(pending) = field(v, &["pendingPaymentMethod", "pending_payment_method"]) {
        if !pending.is_null() {
            out.push_str("Pending payment method:\n");
            if let Some(url) = field_str(pending, &["achMicrodepositHostedVerificationUrl"]) {
                out.push_str(&format!("  ACH verification URL: {url}\n"));
            } else {
                out.push_str(&format!("  {pending}\n"));
            }
        }
    }
    out.push_str("Note: set_default_payment_method chooses which method is charged for top-ups.\n");
    out
}

pub fn fmt_spending_limits(v: &Value) -> String {
    let sl = field(v, &["spendingLimits", "spending_limits"]).unwrap_or(v);
    let soft = money_cell(field(sl, &["softSl", "soft_sl"]));
    let eff = money_cell(field(sl, &["effectiveSl", "effective_sl"]));
    let hard = money_cell(field(sl, &["effectiveHardSl", "effective_hard_sl"]));
    let hard_auto = money_cell(field(sl, &["hardSlAuto", "hard_sl_auto"]));
    let hard_over = money_cell(field(sl, &["hardSlOverride", "hard_sl_override"]));
    let next = money_cell(field(
        sl,
        &["nextBpDesiredSoftSl", "next_bp_desired_soft_sl"],
    ));

    let mut out = String::from("Postpaid spending limits (USD)\n\n");
    out.push_str(&format!("  Soft limit (user-set):     {soft}\n"));
    out.push_str(&format!("  Effective soft limit:      {eff}\n"));
    out.push_str(&format!("  Effective hard limit:      {hard}\n"));
    out.push_str(&format!("  Hard limit (auto):         {hard_auto}\n"));
    if hard_over != "-" {
        out.push_str(&format!("  Hard limit override:       {hard_over}\n"));
    }
    if next != "-" {
        out.push_str(&format!("  Next period soft limit:    {next}\n"));
    }
    out.push_str(
        "\nNotes:\n\
         - Soft limit caps postpaid usage; API rejects new work after it is hit.\n\
         - Prepaid credits are always spent first and are NOT capped by the soft limit.\n\
         - Set soft limit to $0.00 for prepaid-only (set_spending_limit limit_cents=0).\n\
         - Hard limit is an xAI safety ceiling (usually not user-writable).\n",
    );
    out
}
