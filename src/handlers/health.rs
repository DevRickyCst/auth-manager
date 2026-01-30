use crate::api::AppResponse;

/// GET /health
/// Simple healthcheck endpoint
pub async fn health() -> AppResponse<serde_json::Value> {
    AppResponse::ok(serde_json::json!({
        "status": "ok"
    }))
}
