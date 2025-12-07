use argh::FromArgs;
use time::macros::format_description;
use tracing::{debug, info};
use tracing_subscriber::fmt::time::UtcTime;

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
#[argh(description = "A brief description of what your program does.")]
#[argh(help_triggers("-h", "--help", "help"))]
pub struct Argz {
    /// hostname or ip
    #[argh(option, short = 'h', default = "default_host()")]
    host: String,

    /// port
    #[argh(option, short = 'p', default = "default_port()")]
    port: String,

    /// data_dir
    #[argh(option, short = 'd', default = "default_data_dir()")]
    data_dir: String,
}

fn main() {
    // Initialize tracing
    //
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

    info!("Starting helioscope");

    let argz: Argz = argh::from_env();
    debug!("Args: {:?}", argz);

    info!("Going to listen on {}:{}", argz.host, argz.port);
    info!("Saving data to {}", argz.data_dir);
}
