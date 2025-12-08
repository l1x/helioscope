use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::{Request, Response, StatusCode};
use std::sync::Arc;
use tracing::{debug, error};

use super::types::{ErrorResponse, HealthResponse, ProbeDataBatch, SuccessResponse};
use crate::http::validate::{max_request_size, validate_request_size};
use crate::store::writer::WriterHandle;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

pub struct ServerState {
    pub writer: WriterHandle,
    pub data_dir: String,
}

pub async fn handle_probe_data(
    req: Request<Incoming>,
    state: Arc<ServerState>,
) -> Response<BoxBody> {
    let whole_body = match req.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            error!("Failed to read request body: {}", e);
            return error_response(StatusCode::BAD_REQUEST, "Failed to read body");
        }
    };

    if let Err(e) = validate_request_size(whole_body.len()) {
        error!("Request size validation failed: {}", e);
        return error_response(StatusCode::PAYLOAD_TOO_LARGE, &e.to_string());
    }

    let batch: ProbeDataBatch = match serde_json::from_slice(&whole_body) {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to parse JSON: {}", e);
            return error_response(StatusCode::BAD_REQUEST, "Invalid JSON");
        }
    };

    debug!("Received batch of {} probe data points", batch.data.len());

    if let Err(e) = state.writer.insert_batch(batch.data).await {
        error!("Failed to queue write: {}", e);
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to queue write");
    }

    json_response(
        StatusCode::ACCEPTED,
        &SuccessResponse {
            status: "accepted".to_string(),
        },
    )
}

pub async fn handle_health() -> Response<BoxBody> {
    json_response(
        StatusCode::OK,
        &HealthResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            max_request_size_bytes: max_request_size(),
        },
    )
}

pub async fn handle_not_found() -> Response<BoxBody> {
    error_response(StatusCode::NOT_FOUND, "Not found")
}

fn json_response<T: serde::Serialize>(status: StatusCode, body: &T) -> Response<BoxBody> {
    let json = serde_json::to_string(body).unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(full_body(&json))
        .unwrap()
}

fn error_response(status: StatusCode, message: &str) -> Response<BoxBody> {
    json_response(
        status,
        &ErrorResponse {
            error: message.to_string(),
        },
    )
}

fn full_body(content: &str) -> BoxBody {
    Full::new(Bytes::from(content.to_string()))
        .map_err(|never| match never {})
        .boxed()
}
