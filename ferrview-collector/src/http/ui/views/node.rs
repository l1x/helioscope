// src/http/ui/views/node.rs

use askama::Template;

use crate::http::ui::{helpers, models::NodeDetails, templates::NodeTemplate};

pub fn render(node: &NodeDetails) -> String {
    let short_id = helpers::shorten_uuid(&node.node_id);
    let display_name = node.hostname.as_deref().unwrap_or(&short_id);

    let template = NodeTemplate {
        node,
        display_name,
        version: env!("CARGO_PKG_VERSION"),
    };

    template.render().unwrap_or_else(|e| {
        tracing::error!("Failed to render node template: {}", e);
        format!("Template error: {}", e)
    })
}
