use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeDataPoint {
    pub node_id: String,
    pub timestamp: String,
    pub probe_type: String,
    pub probe_name: String,
    pub probe_value: String,
}
