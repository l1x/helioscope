// src/http/ui/views/home.rs

use crate::http::ui::{components, helpers, layout, models::NodeSummary, styles};

pub fn render(nodes: &[NodeSummary]) -> String {
    let extra_styles = format!(
        "{}{}{}{}",
        styles::HEADER,
        styles::STATS,
        styles::NODE_CARDS,
        styles::INFO_ROWS,
    );

    let header_html = format!(
        r#"        <header>
            <div class="header-content">
                <div class="header-left">
                    <h1>ferrview #</h1>
                </div>
                <div class="header-right">
                    {}
                    {}
                </div>
            </div>
        </header>"#,
        components::stat_card(&nodes.len().to_string(), "Active Nodes"),
        components::stat_card("✓", "Running"),
    );

    let nodes_html = if nodes.is_empty() {
        r#"        <div class="no-data">
            <p>No nodes detected yet. Waiting for metrics...</p>
        </div>"#
            .to_string()
    } else {
        let cards: String = nodes.iter().map(render_node_card).collect();
        format!(
            r#"        <div class="nodes-grid">
{}
        </div>"#,
            cards
        )
    };

    let content = format!("{}\n{}", header_html, nodes_html);
    layout::render("Dashboard", &content, &extra_styles)
}

fn render_node_card(node: &NodeSummary) -> String {
    let short_id = helpers::shorten_uuid(&node.node_id);
    let hostname = node.hostname.as_deref().unwrap_or(&short_id);
    let arch = node.cpu_arch.as_deref().unwrap_or("N/A");
    let cpu = node.cpu_cores.as_deref().unwrap_or("N/A");
    let memory = node
        .memory_total_gb
        .map(helpers::format_memory_gb)
        .unwrap_or_else(|| "N/A".to_string());
    let sensors = node.temp_sensors.as_deref().unwrap_or("N/A");
    let max_temp = node
        .max_temp_celsius
        .map(|t| format!("{:.1}°C", t))
        .unwrap_or_else(|| "N/A".to_string());
    let last_seen = node
        .last_seen
        .as_ref()
        .map(|t| helpers::format_timestamp(t))
        .unwrap_or_else(|| "N/A".to_string());

    format!(
        r#"            <div class="node-card">
                <div class="node-header">
                    <h3>{}</h3>
                    <p class="subtitle">{}</p>
                    <span class="status-badge">Active</span>
                </div>
                <div class="node-info">
                    {}
                    {}
                    {}
                    {}
                    {}
                    {}
                </div>
                <div class="node-actions">
                    {}
                </div>
            </div>
"#,
        hostname,
        arch,
        components::info_row("Full ID", &node.node_id),
        components::info_row("CPU Cores", cpu),
        components::info_row("Memory", &memory),
        components::info_row("Temp Sensors", sensors),
        components::info_row("Max Temp", &max_temp),
        components::info_row("Last Seen", &last_seen),
        components::button(
            &format!("/ui/node/{}", node.node_id),
            "View Node",
            "primary"
        ),
    )
}
