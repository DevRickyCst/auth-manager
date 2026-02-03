// src/app.rs

use axum::{
    Router,
    extract::Extension,
    routing::{delete, get, post},
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::auth::jwt::JwtManager;
use crate::auth::services::AuthService;
use crate::handlers::auth::{login, logout, refresh_token, register};
use crate::handlers::health::health;
use crate::handlers::user::{change_password, delete_user, get_current_user, get_user_by_id};

/// Configure les routes d'authentification
pub fn auth_routes(jwt_manager: JwtManager) -> Router {
    let auth_service = Arc::new(AuthService::new(jwt_manager.clone()));

    // Public endpoints (state: AuthService)
    let public = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .with_state(auth_service.clone());

    // Protected endpoints (state: JwtManager) using AuthClaims
    let protected = Router::new()
        .route("/logout", post(logout))
        .with_state(jwt_manager)
        .layer(Extension(auth_service));

    public.merge(protected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use lambda_http::tower::ServiceExt; // for oneshot

    fn test_jwt() -> JwtManager {
        JwtManager::new("test_secret_for_auth_routes")
    }

    #[tokio::test]
    async fn test_logout_requires_authorization() {
        let jwt = test_jwt();
        let app = auth_routes(jwt);

        let req = Request::builder()
            .uri("/logout")
            .method("POST")
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_logout_success() {
        let jwt = test_jwt();

        // Create a user to generate a token
        use crate::auth::password::PasswordManager;
        use crate::db::models::user::NewUser;
        use crate::db::repositories::user_repository::UserRepository;

        let hash = PasswordManager::hash("OldPass123!").expect("hash");
        let new_user = NewUser {
            email: format!("logout_test_{}@example.com", uuid::Uuid::new_v4()),
            username: "logout_user".to_string(),
            password_hash: Some(hash),
        };
        let user = UserRepository::create(&new_user).expect("create user");
        let token = jwt.generate_token(user.id, 1).expect("token");

        let app = auth_routes(jwt);

        let req = Request::builder()
            .uri("/logout")
            .method("POST")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let _ = UserRepository::delete(user.id);
    }
}

/// Configure les routes utilisateur (exemple)
pub fn user_routes(jwt_manager: JwtManager) -> Router {
    // Service pour les handlers
    let auth_service = Arc::new(AuthService::new(jwt_manager.clone()));

    Router::new()
        .route("/me", get(get_current_user))
        .route("/{id}", get(get_user_by_id))
        .route("/{id}", delete(delete_user))
        .route("/{id}/change-password", post(change_password))
        // Fournit JwtManager en state pour l'extracteur AuthClaims
        .with_state(jwt_manager)
        // Fournit le service en extension pour les handlers
        .layer(Extension(auth_service))
}

/// Construit l'application complète
pub fn build_router(jwt_manager: JwtManager) -> Router {
    let router = Router::new()
        .route("/health", get(health))
        .nest("/auth", auth_routes(jwt_manager.clone()))
        .nest("/users", user_routes(jwt_manager))
        // Middleware global de tracing
        .layer(TraceLayer::new_for_http());

    router
}
