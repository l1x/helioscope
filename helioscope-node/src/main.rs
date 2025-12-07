use argh::FromArgs;
use serde::Deserialize;
use std::fs;
use sysinfo::System;
use time::macros::format_description;
use tracing::{debug, info};
use tracing_subscriber::fmt::time::UtcTime;

use crate::probes::sysinfo::{cpu, mem, temp};

mod probes;

fn default_config_file() -> String {
    String::from("helioscope-node.toml")
}

#[derive(FromArgs, Debug)]
#[argh(description = "A brief description of what your program does.")]
#[argh(help_triggers("-h", "--help", "help"))]
pub struct Argz {
    /// config file location
    #[argh(option, default = "default_config_file()")]
    config_file: String,
}

#[derive(Debug, Deserialize)]
pub struct ProbesConfig {
    pub sysinfo: SysinfoProbes,
    // Future: other probe sources
    // pub something_else: SomethingElseProbes,
}

#[derive(Debug, Deserialize)]
pub struct SysinfoProbes {
    pub cpu: bool,
    pub memory: bool,
    pub disk: bool,
    pub network: bool,
    pub temperature: bool,
    pub static_info: bool,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub node_id: String,
    pub metrics_collector_addr: String,
    pub probes: ProbesConfig,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
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
    info!("Config file is read from: {}", argz.config_file);

    let config = Config::load(&argz.config_file).expect("Failed to load helioscope.toml");

    debug!("Config: {:?}", config);

    let mut sys = System::new_all();
    sys.refresh_all();

    if config.probes.sysinfo.cpu {
        let cpu_data = cpu::probe_cpu(&sys, &config.node_id);
        debug!("{:?}", cpu_data);
    }

    if config.probes.sysinfo.memory {
        mem::probe_memory(&sys);
    }

    if config.probes.sysinfo.temperature {
        temp::probe_temperature();
    }

    info!("Helioscope complete");
}
