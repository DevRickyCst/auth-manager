use serde::Serialize;

/// Response d'erreur publique de l'API
#[derive(Debug, Serialize, Clone)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}
