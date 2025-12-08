use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{debug, error, info};

use super::handlers::{self, ServerState};
use crate::store::errors::StoreError;
use crate::store::writer::WriterHandle;

pub struct HttpServer {
    addr: SocketAddr,
    state: Arc<ServerState>,
}

impl HttpServer {
    pub fn new(
        host: &str,
        port: &str,
        writer: WriterHandle,
        data_dir: String,
    ) -> Result<Self, StoreError> {
        let addr: SocketAddr = format!("{}:{}", host, port)
            .parse()
            .map_err(|e| StoreError::InvalidQuery(format!("Invalid address: {}", e)))?;

        let state = Arc::new(ServerState { writer, data_dir });

        Ok(Self { addr, state })
    }

    pub async fn run(self) -> Result<(), StoreError> {
        let listener = TcpListener::bind(self.addr).await.map_err(StoreError::Io)?;

        info!("HTTP server listening on {}", self.addr);

        loop {
            let (stream, remote_addr) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    continue;
                }
            };

            let io = TokioIo::new(stream);
            let state = Arc::clone(&self.state);

            tokio::spawn(async move {
                let service = service_fn(move |req| {
                    let state = Arc::clone(&state);
                    handle_request(req, state, remote_addr)
                });

                if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                    error!("Error serving connection from {}: {}", remote_addr, e);
                }
            });
        }
    }
}

async fn handle_request(
    req: Request<Incoming>,
    state: Arc<ServerState>,
    remote_addr: SocketAddr,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    debug!(
        "Request from {}: {} {}",
        remote_addr,
        req.method(),
        req.uri()
    );

    let path = req.uri().path();
    let response = match (req.method(), path) {
        (&Method::POST, "/api/v1/probe") => handlers::handle_probe_data(req, state.clone()).await,
        (&Method::GET, "/health") => handlers::handle_health().await,
        (&Method::GET, "/ui") => super::ui::handle_ui_home(&state.data_dir).await,
        (&Method::GET, _) if path.starts_with("/ui/node/") => {
            handle_node_chart_request(path, &state.data_dir).await
        }
        _ => handlers::handle_not_found().await,
    };

    Ok(response)
}

async fn handle_node_chart_request(
    path: &str,
    data_dir: &str,
) -> Response<BoxBody<Bytes, hyper::Error>> {
    // Parse path: /ui/node/{node_id}/{chart_type}.svg
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() < 5 {
        return handlers::handle_not_found().await;
    }

    let node_id = parts[3];
    let chart_file = parts[4];

    // Parse hours from query string (default to 24)
    let hours = 24u32; // TODO: parse from query params

    match chart_file {
        "cpu.svg" => super::charts::handle_cpu_chart(node_id, hours, data_dir).await,
        "memory.svg" => super::charts::handle_memory_chart(node_id, hours, data_dir).await,
        "temperature.svg" => {
            super::charts::handle_temperature_chart(node_id, hours, data_dir).await
        }
        _ => handlers::handle_not_found().await,
    }
}
