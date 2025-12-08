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

    let stats_html = format!(
        r#"            <div class="stats">
                {}
                {}
            </div>"#,
        components::stat_card(&nodes.len().to_string(), "Active Nodes"),
        components::stat_card("✓", "Collector Running"),
    );

    let header_html = format!(
        r#"        <header>
            <h1>🌞 Helioscope</h1>
            <p class="subtitle">System Monitoring Dashboard</p>
{}
        </header>"#,
        stats_html
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
    let cpu = node.cpu_cores.as_deref().unwrap_or("N/A");
    let memory = node
        .memory_total_gb
        .map(helpers::format_memory_gb)
        .unwrap_or_else(|| "N/A".to_string());
    let sensors = node.temp_sensors.as_deref().unwrap_or("N/A");
    let last_seen = node
        .last_seen
        .as_ref()
        .map(|t| helpers::format_timestamp(t))
        .unwrap_or_else(|| "N/A".to_string());

    format!(
        r#"            <div class="node-card">
                <div class="node-header">
                    <h3>Node: {}</h3>
                    <span class="status-badge">Active</span>
                </div>
                <div class="node-info">
                    {}
                    {}
                    {}
                    {}
                    {}
                </div>
                <div class="node-actions">
                    {}
                    {}
                    {}
                </div>
            </div>
"#,
        short_id,
        components::info_row("Full ID", &node.node_id),
        components::info_row("CPU Cores", cpu),
        components::info_row("Memory", &memory),
        components::info_row("Temp Sensors", sensors),
        components::info_row("Last Seen", &last_seen),
        components::button(
            &format!("/ui/node/{}", node.node_id),
            "View Dashboard",
            "primary"
        ),
        components::button(
            &format!("/ui/node/{}/cpu.svg", node.node_id),
            "CPU Chart",
            "secondary"
        ),
        components::button(
            &format!("/ui/node/{}/memory.svg", node.node_id),
            "Memory Chart",
            "secondary"
        ),
    )
}
