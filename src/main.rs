use serde::Deserialize;
use std::fs;
use sysinfo::System;
use time::macros::format_description;
use tracing::{debug, info};
use tracing_subscriber;
use tracing_subscriber::fmt::time::UtcTime;

use crate::probes::sysinfo::{cpu, mem, temp};

mod probes;

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
        .with_max_level(tracing::Level::INFO)
        .with_timer(timer)
        .init();

    info!("Starting helioscope");

    let config = Config::load("helioscope.toml").expect("Failed to load helioscope.toml");

    debug!("Config: {:?}", config);

    let mut sys = System::new_all();
    sys.refresh_all();

    if config.probes.sysinfo.cpu {
        cpu::probe_cpu(&sys);
    }

    if config.probes.sysinfo.memory {
        mem::probe_memory(&sys);
    }

    if config.probes.sysinfo.temperature {
        temp::probe_temperature();
    }

    info!("Helioscope complete");
}
