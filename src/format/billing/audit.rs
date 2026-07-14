//! Audit event line formatting.

use serde_json::Value;

use crate::util::field_str;

pub fn fmt_audit_event(e: &Value) -> String {
    let time = field_str(e, &["eventTime", "time", "createTime", "timestamp"])
        .unwrap_or_else(|| "-".into());
    let user = e
        .get("user")
        .and_then(|u| field_str(u, &["email"]).or_else(|| field_str(u, &["givenName"])))
        .or_else(|| {
            e.get("actor")
                .and_then(|a| a.get("user"))
                .and_then(|u| field_str(u, &["email"]))
        })
        .or_else(|| field_str(e, &["userId"]))
        .unwrap_or_else(|| "-".into());
    let op = field_str(e, &["operation"]).unwrap_or_default();
    let etype = field_str(e, &["entityType", "entity_type"]).unwrap_or_default();
    let ename = field_str(e, &["entityName", "entity_name"]).unwrap_or_default();
    let mut desc = field_str(e, &["description"]).unwrap_or_default();
    if desc.is_empty() {
        let mut parts = Vec::new();
        if !op.is_empty() {
            parts.push(op.trim_start_matches("OPERATION_").to_string());
        }
        if !etype.is_empty() {
            parts.push(etype);
        }
        if !ename.is_empty() {
            parts.push(ename);
        }
        if let Some(pk) = e.get("primaryKey").and_then(Value::as_array) {
            let keys: Vec<String> = pk
                .iter()
                .filter_map(Value::as_str)
                .map(|s| s.to_string())
                .collect();
            if !keys.is_empty() {
                parts.push(keys.join(","));
            }
        }
        desc = if parts.is_empty() {
            field_str(e, &["eventId", "event_id"]).unwrap_or_else(|| "-".into())
        } else {
            parts.join(" · ")
        };
    }
    format!("{time} | {user} | {desc}")
}
