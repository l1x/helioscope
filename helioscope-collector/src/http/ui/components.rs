// src/http/ui/components.rs

/// Render a button/link
pub fn button(href: &str, text: &str, variant: &str) -> String {
    format!(
        r#"<a href="{}" class="btn btn-{}">{}</a>"#,
        href, variant, text
    )
}

/// Render an info row (label: value)
pub fn info_row(label: &str, value: &str) -> String {
    format!(
        r#"<div class="info-row">
    <span class="info-label">{}</span>
    <span class="info-value">{}</span>
</div>"#,
        label, value
    )
}

/// Render a stat card
pub fn stat_card(value: &str, label: &str) -> String {
    format!(
        r#"<div class="stat-card">
    <div class="stat-value">{}</div>
    <div class="stat-label">{}</div>
</div>"#,
        value, label
    )
}

/// Render a meta item (for node details page)
pub fn meta_item(label: &str, value: &str) -> String {
    format!(
        r#"<div class="meta-item">
    <div class="meta-label">{}</div>
    <div class="meta-value">{}</div>
</div>"#,
        label, value
    )
}

/// Render a chart card with embedded image
pub fn chart_card(title: &str, img_src: &str, alt: &str) -> String {
    format!(
        r#"<div class="chart-card">
    <h3>{}</h3>
    <div class="chart-container">
        <img src="{}" alt="{}" loading="lazy">
    </div>
</div>"#,
        title, img_src, alt
    )
}
