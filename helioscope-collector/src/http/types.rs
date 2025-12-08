use serde::{Deserialize, Serialize};

use helioscope_common::ProbeDataPoint;

/// Request payload for probe data submission
#[derive(Debug, Deserialize)]
pub struct ProbeDataBatch {
    pub data: Vec<ProbeDataPoint>,
}

/// Standard success response
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub status: String,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub max_request_size_bytes: usize,
}

/// Standard error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "0.2.0".to_string(),
            max_request_size_bytes: 10485760,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"healthy\""));
        assert!(json.contains("\"version\":\"0.2.0\""));
        assert!(json.contains("\"max_request_size_bytes\":10485760"));
    }

    #[test]
    fn test_health_response_structure() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            max_request_size_bytes: 5242880,
        };

        assert_eq!(response.status, "healthy");
        assert_eq!(response.version, "1.0.0");
        assert_eq!(response.max_request_size_bytes, 5242880);
    }
}
