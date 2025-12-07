use argh::FromArgs;
use time::macros::format_description;
use tracing::{debug, error, info};
use tracing_subscriber::fmt::time::UtcTime;

mod http;
mod store;

use store::writer::WriterService;

fn default_host() -> String {
    String::from("localhost")
}

fn default_port() -> String {
    String::from("8080")
}

fn default_data_dir() -> String {
    String::from("data")
}

#[derive(FromArgs, Debug)]
#[argh(description = "Helioscope metrics collector")]
#[argh(help_triggers("-h", "--help", "help"))]
pub struct Argz {
    /// hostname or ip
    #[argh(option, short = 'l', default = "default_host()")]
    host: String,

    /// port
    #[argh(option, short = 'p', default = "default_port()")]
    port: String,

    /// data directory
    #[argh(option, short = 'd', default = "default_data_dir()")]
    data_dir: String,
}

#[tokio::main]
async fn main() {
    let timer = UtcTime::new(format_description!(
        "[year]-[month padding:zero]-[day padding:zero]T[hour padding:zero]:[minute padding:zero]:[second padding:zero]Z"
    ));

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_timer(timer)
        .init();

    info!("Starting helioscope-collector");

    let argz: Argz = argh::from_env();
    debug!("Args: {:?}", argz);

    // Initialize database writer service
    let (writer_service, writer_handle) = match WriterService::new(&argz.data_dir).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    // Spawn writer task
    let writer_handle_clone = writer_handle.clone();
    let writer_task = tokio::spawn(async move {
        writer_service.run().await;
    });

    // Create and run HTTP server
    let server = match http::server::HttpServer::new(&argz.host, &argz.port, writer_handle) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create HTTP server: {}", e);
            std::process::exit(1);
        }
    };

    info!("Listening on {}:{}", argz.host, argz.port);
    info!("Data directory: {}", argz.data_dir);

    let server_task = tokio::spawn(async move {
        if let Err(e) = server.run().await {
            error!("HTTP server error: {}", e);
        }
    });

    // Wait for Ctrl+C
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
        _ = server_task => {
            info!("Server task completed");
        }
    }

    // Graceful shutdown
    info!("Shutting down...");
    if let Err(e) = writer_handle_clone.shutdown().await {
        error!("Error shutting down writer: {}", e);
    }

    if let Err(e) = writer_task.await {
        error!("Error waiting for writer task: {}", e);
    }

    info!("Shutdown complete");
}
