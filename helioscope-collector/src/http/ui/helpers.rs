// src/http/ui/helpers.rs

use crate::store::queries::MetricDataPoint;
use std::collections::HashMap;

/// Shorten UUID for display (first 8 characters)
pub fn shorten_uuid(uuid: &str) -> String {
    if uuid.len() > 8 {
        format!("{}...", &uuid[..8])
    } else {
        uuid.to_string()
    }
}

/// Format ISO timestamp for display (YYYY-MM-DD HH:MM:SS)
pub fn format_timestamp(timestamp: &str) -> String {
    if timestamp.len() >= 19 {
        format!("{} {}", &timestamp[..10], &timestamp[11..19])
    } else {
        timestamp.to_string()
    }
}

/// Format bytes as GB
pub fn format_memory_gb(gb: f64) -> String {
    format!("{:.1} GB", gb)
}

/// Parse ISO timestamp to Unix timestamp
pub fn parse_timestamp(timestamp_str: &str) -> Result<i64, ()> {
    time::OffsetDateTime::parse(
        timestamp_str,
        &time::format_description::well_known::Rfc3339,
    )
    .map(|dt| dt.unix_timestamp())
    .map_err(|_| ())
}

/// Get current date in YYYY-MM-DD format
pub fn current_date() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02}",
        now.year(),
        now.month() as u8,
        now.day()
    )
}

/// Group metrics by extracting index from pattern
/// e.g., "cpu_core_0_usage_percent" → "Core 0"
pub fn group_metrics_by_index(
    metrics: &[MetricDataPoint],
    prefix: &str,
) -> HashMap<String, Vec<(i64, f64)>> {
    let mut map: HashMap<String, Vec<(i64, f64)>> = HashMap::new();

    for metric in metrics {
        if let Some(index) = extract_index(&metric.probe_name, prefix) {
            let label = if prefix.contains("cpu") {
                format!("Core {}", index)
            } else if prefix.contains("sensor") {
                format!("Sensor {}", index)
            } else {
                format!("#{}", index)
            };

            if let Ok(timestamp) = parse_timestamp(&metric.timestamp) {
                if let Ok(value) = metric.probe_value.parse::<f64>() {
                    map.entry(label).or_default().push((timestamp, value));
                }
            }
        }
    }

    map
}

fn extract_index(name: &str, prefix: &str) -> Option<u32> {
    let without_prefix = name.strip_prefix(prefix)?;
    without_prefix.split('_').next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shorten_uuid() {
        assert_eq!(shorten_uuid("d6f0a1c9-a494-4567"), "d6f0a1c9...");
        assert_eq!(shorten_uuid("short"), "short");
    }

    #[test]
    fn test_format_timestamp() {
        assert_eq!(
            format_timestamp("2024-12-08T09:41:30Z"),
            "2024-12-08 09:41:30"
        );
    }

    #[test]
    fn test_parse_timestamp() {
        assert!(parse_timestamp("2024-12-08T09:41:30Z").is_ok());
        assert!(parse_timestamp("invalid").is_err());
    }
}
