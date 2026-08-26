//! Cents parsing and money display.

use serde_json::Value;

use crate::util::cents_signed_to_dollars;

pub fn parse_cents(v: Option<&Value>) -> Option<i64> {
    let v = v?;
    if let Some(n) = v.as_i64() {
        return Some(n);
    }
    if let Some(n) = v.as_u64() {
        return Some(n as i64);
    }
    if let Some(s) = v.as_str() {
        return s.parse().ok();
    }
    if let Some(inner) = v.get("val") {
        return parse_cents(Some(inner));
    }
    None
}

pub fn money_cell(v: Option<&Value>) -> String {
    match parse_cents(v) {
        Some(c) => cents_signed_to_dollars(c),
        None => "-".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_nested_cents() {
        assert_eq!(parse_cents(Some(&json!({"val": "1413"}))), Some(1413));
        assert_eq!(parse_cents(Some(&json!("6971"))), Some(6971));
    }
}
