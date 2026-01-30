use std::env;
mod app;
mod api;
mod auth;
mod db;
mod handlers;
mod error;

use tracing;

use app::build_router;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod docs;


pub async fn setup_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("debug"))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// ----------------- Main -----------------

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    let jwt_manager = auth::jwt::JwtManager::new(
        &env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string())
    );

    // 2. Créer l'AuthService
    let app = build_router(jwt_manager);

    if env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
        // Lambda
        lambda_http::run(app).await
    } else {
        let addr = "0.0.0.0:3000";
        setup_logging().await;
        let app = app.layer(TraceLayer::new_for_http());
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("🚀 Server running at http://{}", addr);
        axum::serve(listener, app).await?;
        
        Ok(())
    }
}
