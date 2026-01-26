// src/handlers/auth.rs

use axum::{
    extract::ConnectInfo,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use std::net::SocketAddr;

use crate::auth::services::AuthService;
use crate::dto::requests::{LoginRequest, RegisterRequest, RefreshTokenRequest};
use crate::dto::responses::{LoginResponse, RefreshTokenResponse, UserResponse};
use crate::error::AppError;

/// POST /auth/register
/// Inscription d'un nouvel utilisateur
pub async fn register(
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let user = AuthService::register(payload)?;
    Ok((StatusCode::CREATED, Json(user)))
}

/// POST /auth/login
/// Connexion d'un utilisateur
pub async fn login(
    Json(payload): Json<LoginRequest>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<Json<LoginResponse>, AppError> {
    // Récupère le User-Agent s'il existe
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Crée le service (normalement injecté via DI)
    let jwt_manager = crate::auth::jwt::JwtManager::new("your_secret_key");
    let service = AuthService::new(jwt_manager);

    let response = service.login(payload, user_agent)?;
    Ok(Json(response))
}

/// POST /auth/refresh
/// Rafraîchissement des tokens
pub async fn refresh_token(
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, AppError> {
    let jwt_manager = crate::auth::jwt::JwtManager::new("your_secret_key");
    let service = AuthService::new(jwt_manager);

    let response = service.refresh_token(payload)?;
    Ok(Json(response))
}

/// POST /auth/logout
/// Déconnexion (optionnel)
pub async fn logout() -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "message": "Logged out successfully" })),
    ))
}