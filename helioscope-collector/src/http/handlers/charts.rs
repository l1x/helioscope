// src/http/handlers/charts.rs

use hyper::StatusCode;
use tracing::{debug, error};

use crate::charts::{ChartData, SvgRenderer, TimeSeries, TimeSeriesChart};
use crate::http::response::{self, BoxBody};
use crate::http::ui::helpers;
use crate::store::reader::ReaderPool;

pub async fn handle_cpu_chart(
    node_id: &str,
    hours: u32,
    reader: &ReaderPool,
) -> (StatusCode, BoxBody) {
    debug!("Generating CPU chart for node {} ({}h)", node_id, hours);

    let metrics = match reader
        .query_node_metrics(node_id, "cpu_core_%_usage_percent", hours)
        .await
    {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to query CPU metrics: {}", e);
            return response::svg_error("Query failed");
        }
    };

    if metrics.is_empty() {
        return response::svg_error("No CPU data available");
    }

    let series_map = helpers::group_metrics_by_index(&metrics, "cpu_core_");

    if series_map.is_empty() {
        return response::svg_error("No CPU data found");
    }

    let mut chart_data = ChartData::new(format!(
        "CPU Usage - Node {}",
        helpers::shorten_uuid(node_id)
    ))
    .with_labels("Time", "Usage (%)");

    for (name, points) in series_map {
        let mut series = TimeSeries::new(name).with_unit("%");
        for (timestamp, value) in points {
            series.add_point(timestamp, value);
        }
        chart_data.add_series(series);
    }

    render_chart(&chart_data)
}

pub async fn handle_memory_chart(
    node_id: &str,
    hours: u32,
    reader: &ReaderPool,
) -> (StatusCode, BoxBody) {
    debug!("Generating memory chart for node {} ({}h)", node_id, hours);

    let used_metrics = match reader
        .query_node_metrics(node_id, "memory_used_bytes", hours)
        .await
    {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to query used memory: {}", e);
            return response::svg_error("Query failed");
        }
    };

    let total_metrics = match reader
        .query_node_metrics(node_id, "memory_total_bytes", hours)
        .await
    {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to query total memory: {}", e);
            return response::svg_error("Query failed");
        }
    };

    if used_metrics.is_empty() {
        return response::svg_error("No memory data available");
    }

    let mut chart_data = ChartData::new(format!(
        "Memory Usage - Node {}",
        helpers::shorten_uuid(node_id)
    ))
    .with_labels("Time", "Memory (GB)");

    // Used memory series
    let mut used_series = TimeSeries::new("Used Memory").with_unit("GB");
    for metric in &used_metrics {
        if let (Ok(timestamp), Ok(value)) = (
            helpers::parse_timestamp(&metric.timestamp),
            metric.probe_value.parse::<f64>(),
        ) {
            used_series.add_point(timestamp, value / 1_073_741_824.0);
        }
    }
    chart_data.add_series(used_series);

    // Total memory series (if available)
    if !total_metrics.is_empty() {
        let mut total_series = TimeSeries::new("Total Memory").with_unit("GB");
        for metric in &total_metrics {
            if let (Ok(timestamp), Ok(value)) = (
                helpers::parse_timestamp(&metric.timestamp),
                metric.probe_value.parse::<f64>(),
            ) {
                total_series.add_point(timestamp, value / 1_073_741_824.0);
            }
        }
        chart_data.add_series(total_series);
    }

    render_chart(&chart_data)
}

pub async fn handle_temperature_chart(
    node_id: &str,
    hours: u32,
    reader: &ReaderPool,
) -> (StatusCode, BoxBody) {
    debug!(
        "Generating temperature chart for node {} ({}h)",
        node_id, hours
    );

    let metrics = match reader
        .query_node_metrics(node_id, "temperature_sensor_%_celsius", hours)
        .await
    {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to query temperature metrics: {}", e);
            return response::svg_error("Query failed");
        }
    };

    if metrics.is_empty() {
        return response::svg_error("No temperature data available");
    }

    let series_map = helpers::group_metrics_by_index(&metrics, "temperature_sensor_");

    if series_map.is_empty() {
        return response::svg_error("No temperature data found");
    }

    let mut chart_data = ChartData::new(format!(
        "Temperature - Node {}",
        helpers::shorten_uuid(node_id)
    ))
    .with_labels("Time", "Temperature (°C)");

    for (name, points) in series_map {
        let mut series = TimeSeries::new(name).with_unit("°C");
        for (timestamp, value) in points {
            series.add_point(timestamp, value);
        }
        chart_data.add_series(series);
    }

    render_chart(&chart_data)
}

fn render_chart(chart_data: &ChartData) -> (StatusCode, BoxBody) {
    let config = TimeSeriesChart::new(800, 400);
    let renderer = SvgRenderer::new(config);

    match renderer.render_to_string(chart_data) {
        Ok(svg) => response::svg(&svg),
        Err(e) => {
            error!("Failed to render chart: {}", e);
            response::svg_error("Render failed")
        }
    }
}
