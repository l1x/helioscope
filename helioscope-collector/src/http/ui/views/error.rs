// src/http/ui/views/error.rs

use crate::http::ui::{components, layout, styles};

pub fn render(title: &str, message: &str) -> String {
    let content = format!(
        r#"        <div class="error-container">
            <h1>⚠️ {}</h1>
            <p>{}</p>
            {}
        </div>"#,
        title,
        message,
        components::button("/ui", "Back to Dashboard", "primary"),
    );

    layout::render("Error", &content, styles::ERROR_PAGE)
}
