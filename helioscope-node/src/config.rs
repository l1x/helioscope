use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub node_id: String,
    pub metrics_collector_addr: String,
    #[serde(default = "default_collection_interval")]
    pub collection_interval_secs: u64,
    pub probes: ProbesConfig,
}

fn default_collection_interval() -> u64 {
    60 // Default: collect every 60 seconds
}

#[derive(Debug, Deserialize)]
pub struct ProbesConfig {
    pub sysinfo: SysinfoProbes,
}

#[derive(Debug, Deserialize)]
pub struct SysinfoProbes {
    pub cpu: bool,
    pub memory: bool,
    pub temperature: bool,
    pub static_info: bool,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Ok(basic_toml::from_str(&content)?)
    }
}
