// src/handlers/auth.rs

use std::sync::Arc;

use auth_manager_api::{
    LoginRequest, PublicLoginResponse, RefreshTokenRequest, RefreshTokenResponse, RegisterRequest,
    UserResponse,
};
use axum::{
    Json,
    extract::Extension,
    http::{HeaderMap, HeaderValue},
};

use crate::auth::extractors::AuthClaims;
use crate::auth::services::AuthService;
use crate::error::AppError;
use crate::response::AppResponse;

/// POST /auth/register
/// Inscription d'un nouvel utilisateur
pub async fn register(
    Json(payload): Json<RegisterRequest>,
) -> Result<AppResponse<UserResponse>, AppError> {
    let user = AuthService::register(payload)?;
    Ok(AppResponse::created(user))
}

/// POST /auth/login
/// Connexion d'un utilisateur
pub async fn login(
    Extension(auth_service): Extension<Arc<AuthService>>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<AppResponse<PublicLoginResponse>, AppError> {
    // Récupère le User-Agent s'il existe
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(str::to_owned);

    let (response, refresh_hash) = auth_service.login(&payload, user_agent)?;

    // Refresh token hash en cookie HttpOnly uniquement — jamais dans le body
    let cookie_val = format!(
        "refresh_token={refresh_hash}; HttpOnly; Secure; SameSite=None; Path=/auth/refresh"
    );
    let mut out_headers = HeaderMap::new();
    out_headers.insert(
        axum::http::header::SET_COOKIE,
        HeaderValue::from_str(&cookie_val)
            .map_err(|_| AppError::internal("Failed to set cookie"))?,
    );

    Ok(AppResponse::ok(PublicLoginResponse::from(response)).with_headers(out_headers))
}

/// POST /auth/refresh
/// Rafraîchissement des tokens
pub async fn refresh_token(
    Extension(auth_service): Extension<Arc<AuthService>>,
    headers: HeaderMap,
) -> Result<AppResponse<RefreshTokenResponse>, AppError> {
    // Read refresh_token hash from Cookie header
    let raw_cookie = headers
        .get(axum::http::header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::validation("Missing Cookie header"))?;

    let refresh_hash = raw_cookie
        .split(';')
        .find_map(|kv| {
            let mut it = kv.trim().splitn(2, '=');
            match (it.next(), it.next()) {
                (Some("refresh_token"), Some(v)) => Some(v.trim().to_string()),
                _ => None,
            }
        })
        .ok_or_else(|| AppError::validation("Missing refresh_token cookie"))?;

    let (response, new_refresh_hash) = auth_service.refresh_token(&RefreshTokenRequest {
        refresh_token: refresh_hash,
    })?;

    let cookie_val = format!(
        "refresh_token={new_refresh_hash}; HttpOnly; Secure; SameSite=None; Path=/auth/refresh"
    );
    let mut out_headers = HeaderMap::new();
    out_headers.insert(
        axum::http::header::SET_COOKIE,
        HeaderValue::from_str(&cookie_val)
            .map_err(|_| AppError::internal("Failed to set cookie"))?,
    );

    Ok(AppResponse::ok(response).with_headers(out_headers))
}

/// POST /auth/logout
/// Déconnexion (optionnel)
pub async fn logout(claims: AuthClaims) -> Result<AppResponse<serde_json::Value>, AppError> {
    AuthService::logout(claims.sub)?;
    Ok(AppResponse::ok(serde_json::json!({
        "message": "Logged out successfully"
    })))
}
