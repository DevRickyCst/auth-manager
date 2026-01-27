// src/handlers/auth.rs

use axum::{
    http::{HeaderMap, HeaderValue, StatusCode},
    Json,
};
use axum::extract::{State, Extension};
use std::sync::Arc;
use crate::auth::services::AuthService;
use crate::dto::requests::{LoginRequest, RegisterRequest, RefreshTokenRequest};
use crate::dto::responses::{RefreshTokenResponse, UserResponse, PublicLoginResponse};
use crate::error::AppError;
use crate::auth::extractors::AuthClaims;

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
    State(auth_service): State<Arc<AuthService>>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, HeaderMap, Json<PublicLoginResponse>), AppError> {
    // Récupère le User-Agent s'il existe
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let (response, refresh_hash) = auth_service.login(payload, user_agent)?;

    // Build Set-Cookie header for refresh token hash
    let cookie_val = format!(
        "refresh_token={}; HttpOnly; Secure; SameSite=Strict; Path=/auth/refresh",
        refresh_hash
    );
    let mut out_headers = HeaderMap::new();
    out_headers.insert(
        axum::http::header::SET_COOKIE,
        HeaderValue::from_str(&cookie_val).map_err(|_| AppError::internal("Failed to set cookie"))?,
    );

    let public = PublicLoginResponse::from(response);
    Ok((StatusCode::OK, out_headers, Json(public)))
}

/// POST /auth/refresh
/// Rafraîchissement des tokens
pub async fn refresh_token(
    State(auth_service): State<Arc<AuthService>>,
    headers: HeaderMap,
) -> Result<Json<RefreshTokenResponse>, AppError> {
    // Read refresh_token hash from Cookie header
    let raw_cookie = headers
        .get(axum::http::header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::validation("Missing Cookie header"))?;

    let refresh_hash = raw_cookie
        .split(';')
        .filter_map(|kv| {
            let mut it = kv.trim().splitn(2, '=');
            match (it.next(), it.next()) {
                (Some("refresh_token"), Some(v)) => Some(v.trim().to_string()),
                _ => None,
            }
        })
        .next()
        .ok_or_else(|| AppError::validation("Missing refresh_token cookie"))?;

    let response = auth_service.refresh_token(RefreshTokenRequest { refresh_token: refresh_hash })?;
    Ok(Json(response))
}

/// POST /auth/logout
/// Déconnexion (optionnel)
pub async fn logout(
    claims: AuthClaims,
    Extension(auth_service): Extension<Arc<AuthService>>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    auth_service.logout(claims.sub)?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "message": "Logged out successfully" })),
    ))
}