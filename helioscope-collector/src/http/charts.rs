// helioscope-collector/src/http/charts.rs

use bytes::Bytes;
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use hyper::{Response, StatusCode};
use std::collections::HashMap;
use tracing::{debug, error};

use crate::charts::{ChartData, SvgRenderer, TimeSeries, TimeSeriesChart};
use crate::store::db::Database;
use crate::store::queries::{MetricDataPoint, query_node_metrics};

/// Handle CPU chart request for a specific node
pub async fn handle_cpu_chart(
    node_id: &str,
    hours: u32,
    data_dir: &str,
) -> Response<BoxBody<Bytes, hyper::Error>> {
    debug!("Generating CPU chart for node {} ({}h)", node_id, hours);

    // Get current date for database filename
    let date = get_current_date();

    // Open database connection
    let mut db = match Database::new_for_date(data_dir, &date).await {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open database: {}", e);
            return error_response("Database error");
        }
    };

    // Query CPU metrics
    let metrics =
        match query_node_metrics(db.conn(), node_id, "cpu_core_%_usage_percent", hours).await {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to query metrics: {}", e);
                return error_response("Query failed");
            }
        };

    if metrics.is_empty() {
        return error_response("No data available");
    }

    // Group metrics by CPU core
    let series_map = group_by_core(metrics);

    if series_map.is_empty() {
        return error_response("No CPU data found");
    }

    // Build chart data
    let mut chart_data = ChartData::new(format!("CPU Usage - Node {}", shorten_uuid(node_id)))
        .with_labels("Time", "Usage (%)");

    for (core_name, points) in series_map {
        let mut series = TimeSeries::new(core_name).with_unit("%");
        for (timestamp, value) in points {
            series.add_point(timestamp, value);
        }
        chart_data.add_series(series);
    }

    // Render to SVG
    let config = TimeSeriesChart::new(800, 400);
    let renderer = SvgRenderer::new(config);

    match renderer.render_to_string(&chart_data) {
        Ok(svg) => svg_response(&svg),
        Err(e) => {
            error!("Failed to render chart: {}", e);
            error_response("Render failed")
        }
    }
}

/// Handle memory chart request for a specific node
pub async fn handle_memory_chart(
    node_id: &str,
    hours: u32,
    data_dir: &str,
) -> Response<BoxBody<Bytes, hyper::Error>> {
    debug!("Generating memory chart for node {} ({}h)", node_id, hours);

    let date = get_current_date();

    let mut db = match Database::new_for_date(data_dir, &date).await {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open database: {}", e);
            return error_response("Database error");
        }
    };

    // Query memory metrics
    let used_metrics =
        match query_node_metrics(db.conn(), node_id, "memory_used_bytes", hours).await {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to query used memory: {}", e);
                return error_response("Query failed");
            }
        };

    let total_metrics =
        match query_node_metrics(db.conn(), node_id, "memory_total_bytes", hours).await {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to query total memory: {}", e);
                return error_response("Query failed");
            }
        };

    if used_metrics.is_empty() {
        return error_response("No memory data available");
    }

    // Build chart data
    let mut chart_data = ChartData::new(format!("Memory Usage - Node {}", shorten_uuid(node_id)))
        .with_labels("Time", "Memory (GB)");

    // Add used memory series
    let mut used_series = TimeSeries::new("Used Memory").with_unit("GB");
    for metric in used_metrics {
        if let Ok(timestamp) = parse_timestamp(&metric.timestamp) {
            if let Ok(value) = metric.probe_value.parse::<f64>() {
                let gb = value / 1_073_741_824.0; // Convert bytes to GB
                used_series.add_point(timestamp, gb);
            }
        }
    }
    chart_data.add_series(used_series);

    // Add total memory series (as reference line)
    if !total_metrics.is_empty() {
        let mut total_series = TimeSeries::new("Total Memory").with_unit("GB");
        for metric in total_metrics {
            if let Ok(timestamp) = parse_timestamp(&metric.timestamp) {
                if let Ok(value) = metric.probe_value.parse::<f64>() {
                    let gb = value / 1_073_741_824.0;
                    total_series.add_point(timestamp, gb);
                }
            }
        }
        chart_data.add_series(total_series);
    }

    // Render to SVG
    let config = TimeSeriesChart::new(800, 400);
    let renderer = SvgRenderer::new(config);

    match renderer.render_to_string(&chart_data) {
        Ok(svg) => svg_response(&svg),
        Err(e) => {
            error!("Failed to render chart: {}", e);
            error_response("Render failed")
        }
    }
}

/// Handle temperature chart request for a specific node
pub async fn handle_temperature_chart(
    node_id: &str,
    hours: u32,
    data_dir: &str,
) -> Response<BoxBody<Bytes, hyper::Error>> {
    debug!(
        "Generating temperature chart for node {} ({}h)",
        node_id, hours
    );

    let date = get_current_date();

    let mut db = match Database::new_for_date(data_dir, &date).await {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open database: {}", e);
            return error_response("Database error");
        }
    };

    // Query temperature metrics
    let metrics =
        match query_node_metrics(db.conn(), node_id, "temperature_sensor_%_celsius", hours).await {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to query metrics: {}", e);
                return error_response("Query failed");
            }
        };

    if metrics.is_empty() {
        return error_response("No temperature data available");
    }

    // Group by sensor
    let series_map = group_by_sensor(metrics);

    if series_map.is_empty() {
        return error_response("No temperature data found");
    }

    // Build chart data
    let mut chart_data = ChartData::new(format!("Temperature - Node {}", shorten_uuid(node_id)))
        .with_labels("Time", "Temperature (°C)");

    for (sensor_name, points) in series_map {
        let mut series = TimeSeries::new(sensor_name).with_unit("°C");
        for (timestamp, value) in points {
            series.add_point(timestamp, value);
        }
        chart_data.add_series(series);
    }

    // Render to SVG
    let config = TimeSeriesChart::new(800, 400);
    let renderer = SvgRenderer::new(config);

    match renderer.render_to_string(&chart_data) {
        Ok(svg) => svg_response(&svg),
        Err(e) => {
            error!("Failed to render chart: {}", e);
            error_response("Render failed")
        }
    }
}

/// Group CPU metrics by core number
fn group_by_core(metrics: Vec<MetricDataPoint>) -> HashMap<String, Vec<(i64, f64)>> {
    let mut map: HashMap<String, Vec<(i64, f64)>> = HashMap::new();

    for metric in metrics {
        // Extract core number from probe_name (e.g., "cpu_core_0_usage_percent")
        if let Some(core_num) = extract_core_number(&metric.probe_name) {
            let core_name = format!("Core {}", core_num);

            if let Ok(timestamp) = parse_timestamp(&metric.timestamp) {
                if let Ok(value) = metric.probe_value.parse::<f64>() {
                    map.entry(core_name)
                        .or_insert_with(Vec::new)
                        .push((timestamp, value));
                }
            }
        }
    }

    map
}

/// Group temperature metrics by sensor number
fn group_by_sensor(metrics: Vec<MetricDataPoint>) -> HashMap<String, Vec<(i64, f64)>> {
    let mut map: HashMap<String, Vec<(i64, f64)>> = HashMap::new();

    for metric in metrics {
        // Extract sensor number from probe_name
        if let Some(sensor_num) = extract_sensor_number(&metric.probe_name) {
            let sensor_name = format!("Sensor {}", sensor_num);

            if let Ok(timestamp) = parse_timestamp(&metric.timestamp) {
                if let Ok(value) = metric.probe_value.parse::<f64>() {
                    map.entry(sensor_name)
                        .or_insert_with(Vec::new)
                        .push((timestamp, value));
                }
            }
        }
    }

    map
}

/// Extract core number from metric name like "cpu_core_0_usage_percent"
fn extract_core_number(metric_name: &str) -> Option<u32> {
    metric_name
        .strip_prefix("cpu_core_")?
        .split('_')
        .next()?
        .parse()
        .ok()
}

/// Extract sensor number from metric name like "temperature_sensor_0_celsius"
fn extract_sensor_number(metric_name: &str) -> Option<u32> {
    metric_name
        .strip_prefix("temperature_sensor_")?
        .split('_')
        .next()?
        .parse()
        .ok()
}

/// Parse ISO 8601 timestamp to Unix timestamp
fn parse_timestamp(timestamp_str: &str) -> Result<i64, ()> {
    time::OffsetDateTime::parse(
        timestamp_str,
        &time::format_description::well_known::Rfc3339,
    )
    .map(|dt| dt.unix_timestamp())
    .map_err(|_| ())
}

/// Get current date in YYYY-MM-DD format for database filename
fn get_current_date() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02}",
        now.year(),
        now.month() as u8,
        now.day()
    )
}

/// Shorten UUID for display (first 8 characters)
fn shorten_uuid(uuid: &str) -> String {
    if uuid.len() > 8 {
        format!("{}...", &uuid[..8])
    } else {
        uuid.to_string()
    }
}

/// Create SVG response
fn svg_response(svg: &str) -> Response<BoxBody<Bytes, hyper::Error>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/svg+xml")
        .header("Cache-Control", "public, max-age=60")
        .body(full_body(svg))
        .unwrap()
}

/// Create error response
fn error_response(message: &str) -> Response<BoxBody<Bytes, hyper::Error>> {
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="400">
            <rect width="800" height="400" fill="#f8f9fa"/>
            <text x="400" y="200" text-anchor="middle" font-family="sans-serif" font-size="16" fill="#dc3545">
                Error: {}
            </text>
        </svg>"##,
        message
    );

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("Content-Type", "image/svg+xml")
        .body(full_body(&svg))
        .unwrap()
}

fn full_body(content: &str) -> BoxBody<Bytes, hyper::Error> {
    Full::new(Bytes::from(content.to_string()))
        .map_err(|never| match never {})
        .boxed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_core_number() {
        assert_eq!(extract_core_number("cpu_core_0_usage_percent"), Some(0));
        assert_eq!(extract_core_number("cpu_core_3_usage_percent"), Some(3));
        assert_eq!(extract_core_number("cpu_core_15_usage_percent"), Some(15));
        assert_eq!(extract_core_number("invalid"), None);
    }

    #[test]
    fn test_extract_sensor_number() {
        assert_eq!(
            extract_sensor_number("temperature_sensor_0_celsius"),
            Some(0)
        );
        assert_eq!(
            extract_sensor_number("temperature_sensor_5_celsius"),
            Some(5)
        );
        assert_eq!(extract_sensor_number("invalid"), None);
    }

    #[test]
    fn test_shorten_uuid() {
        assert_eq!(
            shorten_uuid("d6f0a1c9-a494-4567-8f8d-091cc532c875"),
            "d6f0a1c9..."
        );
        assert_eq!(shorten_uuid("short"), "short");
    }

    #[test]
    fn test_parse_timestamp() {
        assert!(parse_timestamp("2024-12-08T09:41:30Z").is_ok());
        assert!(parse_timestamp("invalid").is_err());
    }
}
