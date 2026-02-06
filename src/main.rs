use std::env;
mod api;
mod app;
mod auth;
mod db;
mod error;
mod handlers;

use app::build_router;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn setup_logging() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Si RUST_LOG n'est pas défini, utiliser ces règles par défaut
        tracing_subscriber::EnvFilter::new(
            "info,auth_manager=debug,hyper_util=warn,tower_http=info",
        )
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// ----------------- Main -----------------

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    // Initialize logging for all environments
    setup_logging().await;
    tracing::info!("Starting auth-manager...");

    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::warn!("JWT_SECRET not set, using default (NOT FOR PRODUCTION!)");
        "default_secret".to_string()
    });

    let jwt_manager = auth::jwt::JwtManager::new(&jwt_secret);
    let app = build_router(jwt_manager);

    if env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
        tracing::info!("Running in Lambda mode");
        lambda_http::run(app).await
    } else {
        tracing::info!("Running in local HTTP server mode");
        let addr = "0.0.0.0:3000";
        let app = app.layer(TraceLayer::new_for_http());
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("🚀 Server running at http://{}", addr);
        axum::serve(listener, app).await?;

        Ok(())
    }
}
