// helioscope-collector/src/http/ui.rs

use bytes::Bytes;
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use hyper::{Response, StatusCode};
use tracing::{debug, error};

use crate::store::db::Database;
use crate::store::queries::{query_all_node_ids, query_latest_node_metrics};

/// Generate the HTML homepage for the UI with dynamic node list
pub async fn handle_ui_home(data_dir: &str) -> Response<BoxBody<Bytes, hyper::Error>> {
    debug!("Generating UI homepage");

    // Get current date for database filename
    let date = get_current_date();

    // Open database connection
    let mut db = match Database::new_for_date(data_dir, &date).await {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open database: {}", e);
            return error_html_response("Database error");
        }
    };

    // Query all node IDs
    let node_ids = match query_all_node_ids(db.conn()).await {
        Ok(ids) => ids,
        Err(e) => {
            error!("Failed to query node IDs: {}", e);
            return error_html_response("Failed to query nodes");
        }
    };

    // Build node info list
    let mut nodes_html = String::new();

    if node_ids.is_empty() {
        nodes_html.push_str(
            r#"
            <div class="no-data">
                <p>No nodes detected yet. Waiting for metrics...</p>
            </div>
        "#,
        );
    } else {
        for node_id in &node_ids {
            // Get latest metrics for this node
            let metrics = match query_latest_node_metrics(db.conn(), node_id).await {
                Ok(m) => m,
                Err(_) => Vec::new(),
            };

            // Extract static info
            let mut cpu_count = String::from("N/A");
            let mut memory_total = String::from("N/A");
            let mut temp_count = String::from("N/A");
            let mut last_seen = String::from("N/A");

            for metric in &metrics {
                if !last_seen.eq("N/A") && metric.timestamp.len() > 0 {
                    last_seen = format_timestamp(&metric.timestamp);
                } else if last_seen.eq("N/A") {
                    last_seen = format_timestamp(&metric.timestamp);
                }

                match metric.probe_name.as_str() {
                    "cpu_core_count" => cpu_count = metric.probe_value.clone(),
                    "memory_total_bytes" => {
                        if let Ok(bytes) = metric.probe_value.parse::<f64>() {
                            let gb = bytes / 1_073_741_824.0;
                            memory_total = format!("{:.1} GB", gb);
                        }
                    }
                    "temperature_sensor_count" => temp_count = metric.probe_value.clone(),
                    _ => {}
                }
            }

            let short_id = shorten_uuid(node_id);

            nodes_html.push_str(&format!(
                r#"
                <div class="node-card">
                    <div class="node-header">
                        <h3>Node: {short_id}</h3>
                        <span class="status-badge status-active">Active</span>
                    </div>
                    <div class="node-info">
                        <div class="info-row">
                            <span class="info-label">Full ID:</span>
                            <span class="info-value node-id-full">{node_id}</span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">CPU Cores:</span>
                            <span class="info-value">{cpu_count}</span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">Memory:</span>
                            <span class="info-value">{memory_total}</span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">Temp Sensors:</span>
                            <span class="info-value">{temp_count}</span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">Last Seen:</span>
                            <span class="info-value">{last_seen}</span>
                        </div>
                    </div>
                    <div class="node-actions">
                        <a href="/ui/node/{node_id}" class="btn btn-primary">View Dashboard</a>
                        <a href="/ui/node/{node_id}/cpu.svg" class="btn btn-secondary" target="_blank">CPU Chart</a>
                        <a href="/ui/node/{node_id}/memory.svg" class="btn btn-secondary" target="_blank">Memory Chart</a>
                    </div>
                </div>
                "#,
                short_id = short_id,
                node_id = node_id,
                cpu_count = cpu_count,
                memory_total = memory_total,
                temp_count = temp_count,
                last_seen = last_seen,
            ));
        }
    }

    let html = format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Helioscope - Monitoring Nodes</title>
    <style>
        :root {{
            --bg-gradient-start: #ff6b35;
            --bg-gradient-end: #f7931e;
            --card-bg: white;
            --text-primary: #2d3748;
            --text-secondary: #718096;
            --border-color: #e2e8f0;
            --accent-color: #ff6b35;
            --accent-hover: #e85a2a;
            --secondary-bg: #f7fafc;
            --secondary-hover: #edf2f7;
        }}

        @media (prefers-color-scheme: dark) {{
            :root {{
                --bg-gradient-start: #1a202c;
                --bg-gradient-end: #2d3748;
                --card-bg: #2d3748;
                --text-primary: #f7fafc;
                --text-secondary: #cbd5e0;
                --border-color: #4a5568;
                --accent-color: #ff6b35;
                --accent-hover: #ff8555;
                --secondary-bg: #1a202c;
                --secondary-hover: #374151;
            }}
        }}

        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            line-height: 1.6;
            color: var(--text-primary);
            background: linear-gradient(135deg, var(--bg-gradient-start) 0%, var(--bg-gradient-end) 100%);
            min-height: 100vh;
            padding: 40px 20px;
        }}

        .container {{
            max-width: 1200px;
            margin: 0 auto;
        }}

        header {{
            background: var(--card-bg);
            border-radius: 12px;
            padding: 30px;
            margin-bottom: 30px;
            box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
        }}

        h1 {{
            color: var(--accent-color);
            margin-bottom: 10px;
            font-size: 2.5em;
        }}

        .subtitle {{
            color: var(--text-secondary);
            font-size: 1.1em;
        }}

        .stats {{
            display: flex;
            gap: 20px;
            margin-top: 20px;
        }}

        .stat-card {{
            background: var(--secondary-bg);
            padding: 15px 20px;
            border-radius: 8px;
            border-left: 4px solid var(--accent-color);
        }}

        .stat-value {{
            font-size: 2em;
            font-weight: bold;
            color: var(--accent-color);
        }}

        .stat-label {{
            color: var(--text-secondary);
            font-size: 0.9em;
        }}

        .nodes-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(500px, 1fr));
            gap: 20px;
        }}

        .node-card {{
            background: var(--card-bg);
            border-radius: 12px;
            padding: 25px;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
            transition: transform 0.2s, box-shadow 0.2s;
        }}

        .node-card:hover {{
            transform: translateY(-4px);
            box-shadow: 0 8px 20px rgba(0, 0, 0, 0.3);
        }}

        .node-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 20px;
            padding-bottom: 15px;
            border-bottom: 1px solid var(--border-color);
        }}

        .node-header h3 {{
            color: var(--text-primary);
            font-size: 1.3em;
        }}

        .status-badge {{
            padding: 5px 12px;
            border-radius: 20px;
            font-size: 0.85em;
            font-weight: 600;
        }}

        .status-active {{
            background: var(--accent-color);
            color: white;
        }}

        .node-info {{
            margin-bottom: 20px;
        }}

        .info-row {{
            display: flex;
            justify-content: space-between;
            padding: 8px 0;
            border-bottom: 1px solid var(--border-color);
        }}

        .info-label {{
            color: var(--text-secondary);
            font-weight: 500;
        }}

        .info-value {{
            color: var(--text-primary);
            font-weight: 600;
        }}

        .node-id-full {{
            font-family: "Courier New", monospace;
            font-size: 0.85em;
        }}

        .node-actions {{
            display: flex;
            gap: 10px;
            flex-wrap: wrap;
        }}

        .btn {{
            padding: 10px 16px;
            border-radius: 6px;
            text-decoration: none;
            font-weight: 500;
            font-size: 0.9em;
            transition: all 0.2s;
            display: inline-block;
        }}

        .btn-primary {{
            background: var(--accent-color);
            color: white;
        }}

        .btn-primary:hover {{
            background: var(--accent-hover);
        }}

        .btn-secondary {{
            background: var(--secondary-bg);
            color: var(--text-primary);
            border: 1px solid var(--border-color);
        }}

        .btn-secondary:hover {{
            background: var(--secondary-hover);
        }}

        .no-data {{
            background: var(--card-bg);
            border-radius: 12px;
            padding: 40px;
            text-align: center;
            color: var(--text-secondary);
        }}

        footer {{
            margin-top: 40px;
            text-align: center;
            color: rgba(255, 255, 255, 0.9);
            opacity: 0.9;
        }}

        footer a {{
            color: rgba(255, 255, 255, 0.9);
            text-decoration: underline;
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>🌞 Helioscope</h1>
            <p class="subtitle">System Monitoring Dashboard</p>
            <div class="stats">
                <div class="stat-card">
                    <div class="stat-value">{node_count}</div>
                    <div class="stat-label">Active Nodes</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">✓</div>
                    <div class="stat-label">Collector Running</div>
                </div>
            </div>
        </header>

        <div class="nodes-grid">
            {nodes_html}
        </div>

        <footer>
            <p>Helioscope v0.2.0 | Built with Rust 🦀</p>
        </footer>
    </div>
</body>
</html>"##,
        node_count = node_ids.len(),
        nodes_html = nodes_html
    );

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(full_body(&html))
        .unwrap()
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

/// Format timestamp for display
fn format_timestamp(timestamp: &str) -> String {
    if timestamp.len() >= 19 {
        // Extract YYYY-MM-DD HH:MM:SS from ISO 8601
        format!("{} {}", &timestamp[..10], &timestamp[11..19])
    } else {
        timestamp.to_string()
    }
}

/// Create error HTML response
fn error_html_response(message: &str) -> Response<BoxBody<Bytes, hyper::Error>> {
    let html = format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Error - Helioscope</title>
    <style>
        body {{
            font-family: sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
            background: #f8f9fa;
            margin: 0;
        }}
        .error {{
            background: white;
            padding: 40px;
            border-radius: 8px;
            box-shadow: 0 4px 12px rgba(0,0,0,0.1);
            text-align: center;
        }}
        h1 {{ color: #dc3545; }}
    </style>
</head>
<body>
    <div class="error">
        <h1>⚠️ Error</h1>
        <p>{}</p>
    </div>
</body>
</html>"##,
        message
    );

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(full_body(&html))
        .unwrap()
}

fn full_body(content: &str) -> BoxBody<Bytes, hyper::Error> {
    Full::new(Bytes::from(content.to_string()))
        .map_err(|never| match never {})
        .boxed()
}
