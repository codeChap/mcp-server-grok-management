//! Usage analytics formatting.

use serde_json::Value;

use crate::util::field;

pub fn fmt_usage(v: &Value, start: &str, end: &str, unit: &str) -> String {
    let series = field(v, &["timeSeries", "time_series"])
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let limit = v
        .get("limitReached")
        .or_else(|| v.get("limit_reached"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let mut out = format!("Usage analytics  {start} → {end}  ({unit})\n");
    if limit {
        out.push_str("WARNING: result cardinality limit reached — partial data.\n");
    }
    if series.is_empty() {
        out.push_str("\nNo time series returned.\n");
        return out;
    }

    // Sum each series
    let mut rows: Vec<(String, f64)> = Vec::new();
    let mut grand = 0.0f64;
    for s in &series {
        let label = field(s, &["groupLabels", "group_labels", "group"])
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(" / ")
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "(ungrouped)".into());
        let mut sum = 0.0f64;
        if let Some(pts) = field(s, &["dataPoints", "data_points"]).and_then(Value::as_array) {
            for p in pts {
                if let Some(vals) = p.get("values").and_then(Value::as_array) {
                    for val in vals {
                        if let Some(n) = val.as_f64() {
                            sum += n;
                        } else if let Some(s) = val.as_str() {
                            if let Ok(n) = s.parse::<f64>() {
                                sum += n;
                            }
                        }
                    }
                }
            }
        }
        grand += sum;
        rows.push((label, sum));
    }
    rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    out.push_str(&format!(
        "\nTotal (USD, summed series): ${grand:.4}\n\nBy group:\n"
    ));
    for (label, sum) in rows.iter().take(40) {
        out.push_str(&format!("  ${sum:>10.4}  {label}\n"));
    }
    if rows.len() > 40 {
        out.push_str(&format!("  … +{} more groups\n", rows.len() - 40));
    }
    out.push_str(
        "\nTip: group_by can be [\"description\"], [\"model\"], etc. when the API supports it.\n",
    );
    out
}
