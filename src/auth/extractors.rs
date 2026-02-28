use axum::extract::FromRequestParts;
use axum::http::{header, request::Parts};

use crate::auth::jwt::{Claims, JwtManager};
use crate::error::AppError;

/// Extracteur d'authentification pour les routes protégées.
/// Valide `Authorization: Bearer <JWT>`, vérifie le token via `JwtManager`,
/// et expose les claims utiles (notamment `sub`).
#[derive(Debug, Clone)]
pub struct AuthClaims {
    pub sub: uuid::Uuid,
    #[expect(
        dead_code,
        reason = "JWT standard claim; available for future token introspection"
    )]
    pub iat: i64,
    #[expect(
        dead_code,
        reason = "JWT standard claim; available for future token introspection"
    )]
    pub exp: i64,
}

impl From<Claims> for AuthClaims {
    fn from(c: Claims) -> Self {
        Self {
            sub: c.sub,
            iat: c.iat,
            exp: c.exp,
        }
    }
}

/// Implémentation de l'extracteur pour un router ayant `JwtManager` comme state.
impl FromRequestParts<JwtManager> for AuthClaims {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        jwt_manager: &JwtManager,
    ) -> Result<Self, Self::Rejection> {
        const BEARER: &str = "Bearer ";

        // Récupère le header Authorization
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .ok_or(AppError::InvalidTokenFormat)?;

        let auth_str = auth_header
            .to_str()
            .map_err(|_| AppError::InvalidTokenFormat)?;

        // Doit être de type Bearer
        if !auth_str.starts_with(BEARER) {
            return Err(AppError::InvalidTokenFormat);
        }

        let token = &auth_str[BEARER.len()..];

        // Vérifie et décode le token
        let claims = jwt_manager
            .verify_token(token)
            .map_err(|_| AppError::unauthorized("Invalid token"))?;

        Ok(AuthClaims::from(claims))
    }
}
