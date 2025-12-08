// src/http/response.rs

use bytes::Bytes;
use http_body_util::{BodyExt, Full, combinators};
use hyper::StatusCode;

pub type BoxBody = combinators::BoxBody<Bytes, hyper::Error>;

/// Create a full body from string content
pub fn full_body(content: &str) -> BoxBody {
    Full::new(Bytes::from(content.to_string()))
        .map_err(|never| match never {})
        .boxed()
}

/// JSON response with status code
pub fn json<T: serde::Serialize>(status: StatusCode, body: &T) -> (StatusCode, BoxBody) {
    let json = serde_json::to_string(body).unwrap_or_else(|_| "{}".to_string());
    (status, full_body(&json))
}

/// JSON error response
pub fn json_error(status: StatusCode, message: &str) -> (StatusCode, BoxBody) {
    json(status, &serde_json::json!({ "error": message }))
}

/// HTML response (200 OK)
pub fn html(content: &str) -> (StatusCode, BoxBody) {
    (StatusCode::OK, full_body(content))
}

/// HTML error response
pub fn html_error(status: StatusCode, content: &str) -> (StatusCode, BoxBody) {
    (status, full_body(content))
}

/// SVG response (200 OK)
pub fn svg(content: &str) -> (StatusCode, BoxBody) {
    (StatusCode::OK, full_body(content))
}

/// SVG error response with embedded error message
pub fn svg_error(message: &str) -> (StatusCode, BoxBody) {
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="400">
    <rect width="800" height="400" fill="#f8f9fa"/>
    <text x="400" y="200" text-anchor="middle" font-family="sans-serif" font-size="16" fill="#dc3545">
        Error: {}
    </text>
</svg>"##,
        message
    );
    (StatusCode::INTERNAL_SERVER_ERROR, full_body(&svg))
}
