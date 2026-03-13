mod app;
mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod response;

use app::build_router;
use config::Config;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup_logging() {
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
    setup_logging();
    tracing::info!("🚀 Starting auth-manager...");

    // Load configuration (auto-détecte l'environnement)
    tracing::info!("📦 Loading configuration...");
    let config = Config::from_env()
        .inspect_err(|e| tracing::error!("❌ Failed to load configuration: {:#}", e))?;
    tracing::info!("✅ Configuration loaded successfully");

    // Initialize database connection pool
    tracing::info!("🔌 Initializing database connection pool...");
    db::connection::init_pool_with_url(&config.database_url).inspect_err(|e| {
        tracing::error!("❌ Failed to initialize database connection pool: {:#}", e);
        tracing::error!("   This is usually caused by:");
        tracing::error!("   1. DATABASE_URL is incorrect");
        tracing::error!("   2. Database is not accessible from Lambda");
        tracing::error!("   3. Credentials are invalid");
        tracing::error!("   4. SSL/TLS issues");
    })?;
    tracing::info!("✅ Database connection pool initialized");

    // Create JWT manager
    let jwt_manager = auth::jwt::JwtManager::new(&config.jwt_secret, config.jwt_expiration_hours);

    // Build router
    let app = build_router(jwt_manager);

    // Run server based on environment (Local → HTTP server, Dev/Prod → Lambda)
    if !config.is_local() {
        tracing::info!(
            "☁️  Running in AWS Lambda mode ({})",
            config.environment.as_str()
        );
        lambda_http::run(app).await
    } else {
        tracing::info!("💻 Running in local HTTP server mode");
        let addr = format!("{}:{}", config.server_host, config.server_port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        tracing::info!("🌐 Server listening on http://{}", addr);
        axum::serve(listener, app).await?;

        Ok(())
    }
}
