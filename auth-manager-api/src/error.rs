use serde::{Deserialize, Serialize};

/// Public API error response format
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}
