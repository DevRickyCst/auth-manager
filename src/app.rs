// src/app.rs

use axum::{
    extract::Extension,
    middleware,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::auth::services::AuthService;
use crate::auth::jwt::JwtManager;
use crate::handlers::auth::{login_handler, register_handler, refresh_token_handler};

/// Configure les routes d'authentification
pub fn auth_routes(auth_service: Arc<AuthService>) -> Router {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/refresh", post(refresh_token_handler))
        .layer(Extension(auth_service))
}

/// Configure les routes utilisateur (exemple)
pub fn user_routes(auth_service: Arc<AuthService>) -> Router {
    Router::new()
        // .route("/me", get(get_user_handler))
        // .route("/:id", get(get_user_by_id_handler))
        // .route("/:id", delete(delete_user_handler))
        .layer(Extension(auth_service))
}

/// Construit l'application complète
pub fn build_router(jwt_manager: JwtManager) -> Router {
    let auth_service = Arc::new(AuthService::new(jwt_manager));

    Router::new()
        .nest("/auth", auth_routes(auth_service.clone()))
        .nest("/users", user_routes(auth_service))
        // Middleware global de tracing
        .layer(TraceLayer::new_for_http())
        // Middleware pour les logs (optionnel)
        // .layer(middleware::from_fn(log_middleware))
}
