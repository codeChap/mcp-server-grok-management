use serde_json::Value;

pub fn field<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    for k in keys {
        if let Some(x) = v.get(*k) {
            if !x.is_null() {
                return Some(x);
            }
        }
    }
    None
}

pub fn field_str(v: &Value, keys: &[&str]) -> Option<String> {
    field(v, keys).and_then(|x| match x {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    })
}

pub fn array_field(v: &Value, keys: &[&str]) -> Vec<Value> {
    if let Some(a) = field(v, keys).and_then(Value::as_array) {
        return a.clone();
    }
    if let Some(a) = v.as_array() {
        return a.clone();
    }
    Vec::new()
}

pub fn pretty(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
}

pub fn cents_to_dollars(cents: u64) -> String {
    let dollars = cents / 100;
    let rem = cents % 100;
    format!("${dollars}.{rem:02}")
}

pub fn cents_signed_to_dollars(cents: i64) -> String {
    if cents < 0 {
        format!("-{}", cents_to_dollars((-cents) as u64))
    } else {
        cents_to_dollars(cents as u64)
    }
}

pub fn normalize_usage_timestamp(s: &str) -> String {
    let mut s = s.trim().to_string();
    if let Some(pos) = s.find('T') {
        s.replace_range(pos..=pos, " ");
    }
    if s.ends_with('Z') {
        s.pop();
    }
    if s.len() > 19 {
        let bytes = s.as_bytes();
        if let Some(idx) = (19..s.len()).find(|&i| bytes[i] == b'+' || bytes[i] == b'-') {
            s.truncate(idx);
        }
    }
    s = s.trim().to_string();
    if s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
        s.push_str(" 00:00:00");
    }
    s
}

pub fn map_time_unit(u: Option<&str>) -> Result<&'static str, String> {
    match u.map(|s| s.trim().to_ascii_uppercase()).as_deref() {
        None | Some("") | Some("DAY") | Some("TIME_UNIT_DAY") => Ok("TIME_UNIT_DAY"),
        Some("HOUR") | Some("TIME_UNIT_HOUR") => Ok("TIME_UNIT_HOUR"),
        Some("MONTH") | Some("TIME_UNIT_MONTH") => Ok("TIME_UNIT_MONTH"),
        Some("WEEK") | Some("TIME_UNIT_CALENDAR_WEEK") => Ok("TIME_UNIT_CALENDAR_WEEK"),
        Some("NONE") | Some("TIME_UNIT_NONE") => Ok("TIME_UNIT_NONE"),
        Some(other) => Err(format!(
            "time_unit must be one of DAY, HOUR, MONTH, WEEK, NONE (got {other})"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cents_formatting() {
        assert_eq!(cents_to_dollars(0), "$0.00");
        assert_eq!(cents_to_dollars(5), "$0.05");
        assert_eq!(cents_to_dollars(500), "$5.00");
        assert_eq!(cents_signed_to_dollars(-2500), "-$25.00");
    }

    #[test]
    fn timestamp_normalization() {
        assert_eq!(
            normalize_usage_timestamp("2025-01-01T00:00:00Z"),
            "2025-01-01 00:00:00"
        );
        assert_eq!(
            normalize_usage_timestamp("2025-01-01"),
            "2025-01-01 00:00:00"
        );
    }
}
