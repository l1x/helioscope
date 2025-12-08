// src/http/ui/models.rs

/// Summary info for a node (used in home page list)
#[derive(Debug, Clone)]
pub struct NodeSummary {
    pub node_id: String,
    pub cpu_cores: Option<String>,
    pub memory_total_gb: Option<f64>,
    pub temp_sensors: Option<String>,
    pub last_seen: Option<String>,
}

/// Detailed info for a single node (used in dashboard)
#[derive(Debug, Clone)]
pub struct NodeDetails {
    pub node_id: String,
    pub hostname: Option<String>,
    pub os_name: Option<String>,
    pub kernel_version: Option<String>,
    pub cpu_arch: Option<String>,
    pub cpu_cores: Option<String>,
    pub memory_total_gb: Option<f64>,
    pub last_seen: Option<String>,
}

impl NodeSummary {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            cpu_cores: None,
            memory_total_gb: None,
            temp_sensors: None,
            last_seen: None,
        }
    }
}

impl NodeDetails {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            hostname: None,
            os_name: None,
            kernel_version: None,
            cpu_arch: None,
            cpu_cores: None,
            memory_total_gb: None,
            last_seen: None,
        }
    }
}
