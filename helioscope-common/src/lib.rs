use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProbeDataPoint {
    pub node_id: String,
    pub timestamp: String,
    pub probe_type: String,
    pub probe_name: String,
    pub probe_value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_probe_data_point() {
        let point = ProbeDataPoint {
            node_id: "test-node".to_string(),
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            probe_type: "sysinfo".to_string(),
            probe_name: "cpu_count".to_string(),
            probe_value: "8".to_string(),
        };

        let json = serde_json::to_string(&point).unwrap();
        assert!(json.contains("test-node"));
        assert!(json.contains("cpu_count"));
    }

    #[test]
    fn test_deserialize_probe_data_point() {
        let json = r#"{
            "node_id": "test-node",
            "timestamp": "2024-01-01T12:00:00Z",
            "probe_type": "sysinfo",
            "probe_name": "cpu_count",
            "probe_value": "8"
        }"#;

        let point: ProbeDataPoint = serde_json::from_str(json).unwrap();
        assert_eq!(point.node_id, "test-node");
        assert_eq!(point.probe_value, "8");
    }

    #[test]
    fn test_round_trip() {
        let original = ProbeDataPoint {
            node_id: "node-123".to_string(),
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            probe_type: "sysinfo".to_string(),
            probe_name: "memory_total".to_string(),
            probe_value: "16777216".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ProbeDataPoint = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }
}
