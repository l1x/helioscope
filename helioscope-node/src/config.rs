// helioscope-node/src/config.rs
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
        Self::from_str(&content)
    }

    // Helper for tests - parses TOML from string
    fn from_str(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(basic_toml::from_str(content)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let toml = r#"
            node_id = "test-node-123"
            metrics_collector_addr = "localhost:8080"
            collection_interval_secs = 30

            [probes.sysinfo]
            cpu = true
            memory = true
            temperature = false
            static_info = true
        "#;

        let config = Config::from_str(toml).unwrap();

        assert_eq!(config.node_id, "test-node-123");
        assert_eq!(config.metrics_collector_addr, "localhost:8080");
        assert_eq!(config.collection_interval_secs, 30);
        assert!(config.probes.sysinfo.cpu);
        assert!(config.probes.sysinfo.memory);
        assert!(!config.probes.sysinfo.temperature);
        assert!(config.probes.sysinfo.static_info);
    }

    #[test]
    fn test_default_collection_interval() {
        let toml = r#"
            node_id = "test-node"
            metrics_collector_addr = "localhost:8080"

            [probes.sysinfo]
            cpu = true
            memory = true
            temperature = true
            static_info = true
        "#;

        let config = Config::from_str(toml).unwrap();

        assert_eq!(config.collection_interval_secs, 60);
    }

    #[test]
    fn test_all_probes_enabled() {
        let toml = r#"
            node_id = "node-all"
            metrics_collector_addr = "collector.example.com:9090"

            [probes.sysinfo]
            cpu = true
            memory = true
            temperature = true
            static_info = true
        "#;

        let config = Config::from_str(toml).unwrap();

        assert!(config.probes.sysinfo.cpu);
        assert!(config.probes.sysinfo.memory);
        assert!(config.probes.sysinfo.temperature);
        assert!(config.probes.sysinfo.static_info);
    }

    #[test]
    fn test_all_probes_disabled() {
        let toml = r#"
            node_id = "node-none"
            metrics_collector_addr = "collector:8080"

            [probes.sysinfo]
            cpu = false
            memory = false
            temperature = false
            static_info = false
        "#;

        let config = Config::from_str(toml).unwrap();

        assert!(!config.probes.sysinfo.cpu);
        assert!(!config.probes.sysinfo.memory);
        assert!(!config.probes.sysinfo.temperature);
        assert!(!config.probes.sysinfo.static_info);
    }

    #[test]
    fn test_missing_required_field() {
        let toml = r#"
            metrics_collector_addr = "localhost:8080"

            [probes.sysinfo]
            cpu = true
            memory = true
            temperature = true
            static_info = true
        "#;

        let result = Config::from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_probes_section() {
        let toml = r#"
            node_id = "test-node"
            metrics_collector_addr = "localhost:8080"
        "#;

        let result = Config::from_str(toml);
        assert!(result.is_err());
    }
}
