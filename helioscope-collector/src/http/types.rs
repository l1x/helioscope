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

/// Standard error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
