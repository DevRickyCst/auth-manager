use axum::{http::StatusCode, Json};

/// GET /health
/// Simple healthcheck endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "OK", body = serde_json::Value)
    )
)]
pub async fn health() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok"
        })),
    )
}
