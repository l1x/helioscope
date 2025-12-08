// src/http/ui/views/node.rs

use crate::http::ui::{components, helpers, layout, models::NodeDetails, styles};

pub fn render(node: &NodeDetails) -> String {
    let extra_styles = format!(
        "{}{}{}",
        styles::HEADER,
        styles::BUTTONS,
        styles::NODE_DASHBOARD,
    );

    let short_id = helpers::shorten_uuid(&node.node_id);

    let hostname = node.hostname.as_deref().unwrap_or("N/A");
    let os = node.os_name.as_deref().unwrap_or("N/A");
    let kernel = node.kernel_version.as_deref().unwrap_or("N/A");
    let arch = node.cpu_arch.as_deref().unwrap_or("N/A");
    let cores = node.cpu_cores.as_deref().unwrap_or("N/A");
    let memory = node
        .memory_total_gb
        .map(helpers::format_memory_gb)
        .unwrap_or_else(|| "N/A".to_string());

    let header = format!(
        r#"        <header>
            <div class="breadcrumb"><a href="/ui">← Back to Dashboard</a></div>
            <h1>Node: {}</h1>
            <p class="node-id-full">{}</p>
            <div class="node-meta">
                {}
                {}
                {}
                {}
                {}
                {}
            </div>
        </header>"#,
        short_id,
        node.node_id,
        components::meta_item("Hostname", hostname),
        components::meta_item("OS", os),
        components::meta_item("Kernel", kernel),
        components::meta_item("CPU Arch", arch),
        components::meta_item("CPU Cores", cores),
        components::meta_item("Memory", &memory),
    );

    let charts = format!(
        r#"        <div class="charts-section">
            <h2>System Metrics (Last 24 Hours)</h2>
            <div class="charts-grid">
                {}
                {}
                {}
            </div>
        </div>"#,
        components::chart_card(
            "CPU Usage",
            &format!("/ui/node/{}/cpu.svg", node.node_id),
            "CPU Usage"
        ),
        components::chart_card(
            "Memory Usage",
            &format!("/ui/node/{}/memory.svg", node.node_id),
            "Memory Usage"
        ),
        components::chart_card(
            "Temperature",
            &format!("/ui/node/{}/temperature.svg", node.node_id),
            "Temperature"
        ),
    );

    let actions = format!(
        r#"        <div class="actions">
            {}
            {}
            {}
        </div>"#,
        components::button(
            &format!("/ui/node/{}/cpu.svg", node.node_id),
            "Download CPU Chart",
            "secondary"
        ),
        components::button(
            &format!("/ui/node/{}/memory.svg", node.node_id),
            "Download Memory Chart",
            "secondary"
        ),
        components::button(
            &format!("/ui/node/{}/temperature.svg", node.node_id),
            "Download Temp Chart",
            "secondary"
        ),
    );

    let content = format!("{}\n{}\n{}", header, charts, actions);
    layout::render(&format!("Node {}", short_id), &content, &extra_styles)
}
