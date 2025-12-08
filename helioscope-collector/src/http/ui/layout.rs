// src/http/ui/layout.rs

use super::styles;

/// Render a complete HTML page
pub fn render(title: &str, content: &str, extra_styles: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - Helioscope</title>
    <style>
{base}
{buttons}
{extra}
    </style>
</head>
<body>
    <div class="container">
{content}
        <footer>
            <p>Helioscope v{version} | Built with Rust 🦀</p>
        </footer>
    </div>
</body>
</html>"#,
        title = title,
        base = styles::BASE,
        buttons = styles::BUTTONS,
        extra = extra_styles,
        content = content,
        version = env!("CARGO_PKG_VERSION"),
    )
}
