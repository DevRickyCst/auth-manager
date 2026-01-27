use axum::{http::StatusCode, Json};

/// GET /health
/// Simple healthcheck endpoint
pub async fn health() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok"
        })),
    )
}
