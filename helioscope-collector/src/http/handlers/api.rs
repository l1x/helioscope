// src/http/handlers/api.rs

use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::{Request, StatusCode};
use tracing::{debug, error};

use crate::http::response::{self, BoxBody};
use crate::http::types::{HealthResponse, ProbeDataBatch, SuccessResponse};
use crate::http::validate::{max_request_size, validate_request_size};
use crate::store::writer::WriterHandle;

pub async fn handle_probe(req: Request<Incoming>, writer: &WriterHandle) -> (StatusCode, BoxBody) {
    let whole_body = match req.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            error!("Failed to read request body: {}", e);
            return response::json_error(StatusCode::BAD_REQUEST, "Failed to read body");
        }
    };

    if let Err(e) = validate_request_size(whole_body.len()) {
        error!("Request size validation failed: {}", e);
        return response::json_error(StatusCode::PAYLOAD_TOO_LARGE, &e.to_string());
    }

    let batch: ProbeDataBatch = match serde_json::from_slice(&whole_body) {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to parse JSON: {}", e);
            return response::json_error(StatusCode::BAD_REQUEST, "Invalid JSON");
        }
    };

    debug!("Received batch of {} probe data points", batch.data.len());

    if let Err(e) = writer.insert_batch(batch.data).await {
        error!("Failed to queue write: {}", e);
        return response::json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to queue write");
    }

    response::json(
        StatusCode::ACCEPTED,
        &SuccessResponse {
            status: "accepted".to_string(),
        },
    )
}

pub async fn handle_health() -> (StatusCode, BoxBody) {
    response::json(
        StatusCode::OK,
        &HealthResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            max_request_size_bytes: max_request_size(),
        },
    )
}

pub async fn handle_not_found() -> (StatusCode, BoxBody) {
    response::json_error(StatusCode::NOT_FOUND, "Not found")
}
