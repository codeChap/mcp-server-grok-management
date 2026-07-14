//! Formatters for API-key related Management API payloads.

use serde_json::Value;

use crate::util::{field, field_str, pretty};

pub fn format_new_secret(v: &Value) -> String {
    if let Some(secret) = field_str(v, &["apiKey", "api_key", "key", "secret"]) {
        format!(
            "NEW API KEY SECRET (shown once, store it now):\n{secret}\n\n\
             Key ID: {}\nName: {}",
            field_str(v, &["apiKeyId", "api_key_id"]).unwrap_or_else(|| "-".into()),
            field_str(v, &["name"]).unwrap_or_else(|| "-".into()),
        )
    } else {
        format!(
            "Could not locate secret field in response — inspect carefully:\n{}",
            pretty(v)
        )
    }
}

pub fn fmt_api_key_row(v: &Value) -> String {
    let id = field_str(v, &["apiKeyId", "api_key_id"]).unwrap_or_else(|| "-".into());
    let name = field_str(v, &["name"]).unwrap_or_else(|| "(unnamed)".into());
    let redacted =
        field_str(v, &["redactedApiKey", "redacted_api_key"]).unwrap_or_else(|| "-".into());
    let disabled = field(v, &["disabled"])
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    let status = if disabled { "disabled" } else { "active" };
    let acls = field(v, &["aclStrings", "acl_strings", "acls"])
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| "-".into());
    let qps = field_str(v, &["qps"]).unwrap_or_default();
    let qpm = field_str(v, &["qpm"]).unwrap_or_default();
    let tpm = field_str(v, &["tpm"]).unwrap_or_default();
    let rates = {
        let mut parts = Vec::new();
        if !qps.is_empty() {
            parts.push(format!("qps={qps}"));
        }
        if !qpm.is_empty() {
            parts.push(format!("qpm={qpm}"));
        }
        if !tpm.is_empty() {
            parts.push(format!("tpm={tpm}"));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!(" [{}]", parts.join(" "))
        }
    };
    format!("{id} | {name} | {status} | {redacted}{rates} | acls=[{acls}]")
}

pub fn format_ip_ranges(v: &Value) -> String {
    let ranges = field(v, &["ipRanges", "ip_ranges"])
        .and_then(|r| {
            if r.is_null() {
                None
            } else {
                field(r, &["ipRanges", "ip_ranges"]).or(Some(r))
            }
        })
        .and_then(Value::as_array);
    match ranges {
        None => "any".into(),
        Some(arr) if arr.is_empty() => "any".into(),
        Some(arr) => arr
            .iter()
            .map(|r| {
                let prefix = field(r, &["prefixLength", "prefix_length"])
                    .and_then(|p| p.as_i64())
                    .unwrap_or(32);
                let addr = r
                    .get("address")
                    .and_then(|a| field_str(a, &["ipv4", "ipv6"]))
                    .unwrap_or_else(|| r.to_string());
                format!("{addr}/{prefix}")
            })
            .collect::<Vec<_>>()
            .join(", "),
    }
}

pub fn collect_model_names(v: &Value) -> Vec<String> {
    let mut names = Vec::new();
    let mut push_name = |m: &Value| {
        if let Some(n) = field_str(m, &["name", "id"]) {
            if !names.contains(&n) {
                names.push(n);
            }
        } else if let Some(s) = m.as_str() {
            let s = s.to_string();
            if !names.contains(&s) {
                names.push(s);
            }
        }
    };

    if let Some(arr) = field(v, &["models"]).and_then(Value::as_array) {
        for m in arr {
            push_name(m);
        }
        return names;
    }

    if let Some(clusters) =
        field(v, &["clusterConfigs", "cluster_configs"]).and_then(Value::as_array)
    {
        for c in clusters {
            for key in [
                "languageModels",
                "language_models",
                "embeddingModels",
                "embedding_models",
                "imageGenerationModels",
                "image_generation_models",
                "audioModels",
                "audio_models",
                "videoGenerationModels",
                "video_generation_models",
            ] {
                if let Some(arr) = c.get(key).and_then(Value::as_array) {
                    for m in arr {
                        push_name(m);
                    }
                }
            }
        }
        return names;
    }

    if let Some(arr) = v.as_array() {
        for m in arr {
            push_name(m);
        }
    }
    names
}

pub fn fmt_validate_key(v: &Value) -> String {
    let name = field_str(v, &["name"]).unwrap_or_else(|| "(unnamed)".into());
    let team = field_str(v, &["teamId", "scopeId", "team_id", "scope_id"])
        .unwrap_or_else(|| "(unknown)".into());
    let scope = field_str(v, &["scope"]).unwrap_or_default();
    let owner = field_str(v, &["ownerUserId", "owner_user_id"]).unwrap_or_default();
    let redacted = field_str(v, &["redactedApiKey", "reactedApiKey"]).unwrap_or_default();

    let mut out = String::from("Management key: VALID\n");
    out.push_str(&format!("Name: {name}\n"));
    out.push_str(&format!("Team ID: {team}\n"));
    if !scope.is_empty() {
        out.push_str(&format!("Scope: {scope}\n"));
    }
    if !owner.is_empty() {
        out.push_str(&format!("Owner user: {owner}\n"));
    }
    if !redacted.is_empty() {
        out.push_str(&format!("Redacted key: {redacted}\n"));
    }

    let acls: Vec<String> = field(v, &["acls"])
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    let billing_write = acls.iter().any(|a| a.contains("BillingWrite"));
    let billing_read = acls
        .iter()
        .any(|a| a.contains("BillingRead") || a.contains("Billing"));
    out.push_str("\nBilling capability:\n");
    out.push_str(&format!(
        "  BillingRead:  {}\n  BillingWrite: {}\n",
        if billing_read {
            "yes"
        } else {
            "not listed — reads may 403"
        },
        if billing_write {
            "yes"
        } else {
            "not listed — set_spending_limit / top_up / set_billing_info need BillingWrite"
        }
    ));

    out.push_str(&format!("\nACLs ({}):\n", acls.len()));
    if acls.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for a in &acls {
            out.push_str(&format!("  - {a}\n"));
        }
    }
    out.push_str(&format!("\nIP ranges: {}\n", format_ip_ranges(v)));
    out
}

pub fn fmt_list_api_keys(team_id: &str, v: &Value) -> String {
    use crate::util::array_field;
    let keys = array_field(v, &["apiKeys", "api_keys"]);
    if keys.is_empty() {
        return format!("No API keys found for team {team_id}.");
    }
    let lines: Vec<String> = keys.iter().map(fmt_api_key_row).collect();
    let mut out = format!(
        "API keys for team {team_id} ({}):\n\n{}\n",
        keys.len(),
        lines.join("\n")
    );
    if let Some(tok) = field_str(v, &["paginationToken", "nextPageToken", "next_page_token"]) {
        out.push_str(&format!("\nNext page token: {tok}\n"));
    } else {
        out.push_str("\n(no more pages)\n");
    }
    out
}

pub fn fmt_list_models(team_id: &str, v: &Value) -> String {
    let names = collect_model_names(v);
    if names.is_empty() {
        return format!("No models listed.\n\nRaw:\n{}", pretty(v));
    }
    let mut out = format!(
        "Models for team {team_id} ({}). Use as ACL \"api-key:model:<name>\":\n\n",
        names.len()
    );
    for n in names {
        out.push_str(&format!("  - {n}\n"));
    }
    out
}

pub fn fmt_list_endpoints(team_id: &str, v: &Value) -> String {
    use crate::util::array_field;
    let acls = array_field(v, &["acls", "endpoints"]);
    if acls.is_empty() {
        return format!("No endpoints.\n\nRaw:\n{}", pretty(v));
    }
    let mut out = format!(
        "Endpoints for team {team_id} ({}). Use as ACL \"api-key:endpoint:<name>\":\n\n",
        acls.len()
    );
    for a in &acls {
        let acl = field_str(a, &["acl", "name", "value"])
            .unwrap_or_else(|| a.as_str().unwrap_or(&a.to_string()).to_string());
        let desc = field_str(a, &["description"]).unwrap_or_default();
        if desc.is_empty() {
            out.push_str(&format!("  - {acl}\n"));
        } else {
            out.push_str(&format!("  - {acl}  — {desc}\n"));
        }
    }
    out
}

pub fn fmt_list_audit(v: &Value) -> String {
    use crate::format::billing::fmt_audit_event;
    use crate::util::array_field;
    let events = array_field(v, &["events"]);
    if events.is_empty() {
        return "No audit events found.".into();
    }
    let lines: Vec<String> = events.iter().map(fmt_audit_event).collect();
    let mut out = format!("Audit events ({}):\n\n{}\n", events.len(), lines.join("\n"));
    if let Some(tok) = field_str(v, &["nextPageToken", "next_page_token"]) {
        out.push_str(&format!("\nNext page token: {tok}\n"));
    } else {
        out.push_str("\n(no more pages)\n");
    }
    out
}

pub fn fmt_propagation(api_key_id: &str, v: &Value) -> String {
    let mut out = format!("Propagation status for {api_key_id}:\n");
    if let Some(map) = field(v, &["icPropagation", "ic_propagation"]).and_then(Value::as_object)
    {
        for (k, val) in map {
            out.push_str(&format!("  {k}: {val}\n"));
        }
    } else {
        out.push_str(&pretty(v));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn api_key_row_format() {
        let v = json!({
            "apiKeyId": "abc-123",
            "name": "prod",
            "disabled": false,
            "redactedApiKey": "xai-...xyz",
            "qpm": 100,
            "aclStrings": ["api-key:model:*"]
        });
        let line = fmt_api_key_row(&v);
        assert!(line.contains("abc-123"));
        assert!(line.contains("active"));
    }

    #[test]
    fn collect_models() {
        let v = json!({
            "clusterConfigs": [{
                "languageModels": [{"name": "grok-4"}],
                "imageGenerationModels": [{"name": "grok-2-image"}]
            }]
        });
        assert_eq!(collect_model_names(&v), vec!["grok-4", "grok-2-image"]);
    }
}
