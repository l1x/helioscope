// src/http/handlers/web.rs

use hyper::StatusCode;
use tracing::{debug, error};

use crate::http::response::{self, BoxBody};
use crate::http::ui::{
    helpers,
    models::{NodeDetails, NodeSummary},
    views,
};
use crate::store::db::Database;
use crate::store::queries::{MetricDataPoint, query_all_node_ids, query_latest_node_metrics};

pub async fn handle_home(data_dir: &str) -> (StatusCode, BoxBody) {
    debug!("Handling home page request");

    let date = helpers::current_date();
    let mut db = match Database::new_for_date(data_dir, &date).await {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open database: {}", e);
            return render_error("Database Error", "Failed to connect to database");
        }
    };

    let node_ids = match query_all_node_ids(db.conn()).await {
        Ok(ids) => ids,
        Err(e) => {
            error!("Failed to query node IDs: {}", e);
            return render_error("Query Error", "Failed to load node list");
        }
    };

    let mut nodes = Vec::with_capacity(node_ids.len());
    for node_id in node_ids {
        let metrics = query_latest_node_metrics(db.conn(), &node_id)
            .await
            .unwrap_or_default();
        nodes.push(build_node_summary(&node_id, &metrics));
    }

    let html = views::home::render(&nodes);
    response::html(&html)
}

pub async fn handle_node_dashboard(node_id: &str, data_dir: &str) -> (StatusCode, BoxBody) {
    debug!("Handling node dashboard for {}", node_id);

    let date = helpers::current_date();
    let mut db = match Database::new_for_date(data_dir, &date).await {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open database: {}", e);
            return render_error("Database Error", "Failed to connect to database");
        }
    };

    let metrics = match query_latest_node_metrics(db.conn(), node_id).await {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to query metrics for {}: {}", node_id, e);
            return render_error("Query Error", "Failed to load node metrics");
        }
    };

    if metrics.is_empty() {
        return render_error("Not Found", &format!("No data found for node {}", node_id));
    }

    let node_details = build_node_details(node_id, &metrics);
    let html = views::node::render(&node_details);
    response::html(&html)
}

fn render_error(title: &str, message: &str) -> (StatusCode, BoxBody) {
    let html = views::error::render(title, message);
    response::html_error(StatusCode::INTERNAL_SERVER_ERROR, &html)
}

fn build_node_summary(node_id: &str, metrics: &[MetricDataPoint]) -> NodeSummary {
    let mut summary = NodeSummary::new(node_id.to_string());

    for metric in metrics {
        if summary.last_seen.is_none() && !metric.timestamp.is_empty() {
            summary.last_seen = Some(metric.timestamp.clone());
        }
        match metric.probe_name.as_str() {
            "cpu_core_count" => summary.cpu_cores = Some(metric.probe_value.clone()),
            "memory_total_bytes" => {
                if let Ok(bytes) = metric.probe_value.parse::<f64>() {
                    summary.memory_total_gb = Some(bytes / 1_073_741_824.0);
                }
            }
            "temperature_sensor_count" => summary.temp_sensors = Some(metric.probe_value.clone()),
            _ => {}
        }
    }

    summary
}

fn build_node_details(node_id: &str, metrics: &[MetricDataPoint]) -> NodeDetails {
    let mut details = NodeDetails::new(node_id.to_string());

    for metric in metrics {
        if details.last_seen.is_none() && !metric.timestamp.is_empty() {
            details.last_seen = Some(metric.timestamp.clone());
        }
        match metric.probe_name.as_str() {
            "system_hostname" => details.hostname = Some(metric.probe_value.clone()),
            "system_os_name" => details.os_name = Some(metric.probe_value.clone()),
            "system_kernel_version" => details.kernel_version = Some(metric.probe_value.clone()),
            "system_cpu_arch" => details.cpu_arch = Some(metric.probe_value.clone()),
            "cpu_core_count" => details.cpu_cores = Some(metric.probe_value.clone()),
            "memory_total_bytes" => {
                if let Ok(bytes) = metric.probe_value.parse::<f64>() {
                    details.memory_total_gb = Some(bytes / 1_073_741_824.0);
                }
            }
            _ => {}
        }
    }

    details
}
