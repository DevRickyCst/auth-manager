// src/error.rs

use auth_manager_api::ErrorResponse;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, Clone, thiserror::Error)]
pub enum AppError {
    // === Erreurs Repository ===
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Already exists: {0}")]
    Duplicate(String),
    #[error("Database error: {0}")]
    DatabaseError(String),

    // === Erreurs d'Authentification ===
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Email already exists")]
    UserAlreadyExists,
    #[error("Invalid refresh token")]
    InvalidRefreshToken,
    #[error("Refresh token expired")]
    RefreshTokenExpired,
    #[error("Invalid email format")]
    InvalidEmail,
    #[error("Password too weak: {0}")]
    WeakPassword(String),

    // === Erreurs de Hashing/Cryptographie ===
    #[error("Password hashing failed: {0}")]
    PasswordHashingFailed(String),
    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),
    #[error("Invalid token format")]
    InvalidTokenFormat,

    // === Erreurs de Validation ===
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    // === Erreurs métier ===
    #[error("Unauthorized: {0}")]
    UnauthorizedAction(String),
    #[error("Too many attempts: {0}")]
    TooManyAttempts(String),

    // === Erreurs internes ===
    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message, internal_detail) = self.get_error_info();

        if let Some(ref detail) = internal_detail {
            tracing::error!(error_code, %status, detail, "Internal server error");
        }

        let body = Json(ErrorResponse {
            error: error_code.to_string(),
            message,
            details: None,
        });

        (status, body).into_response()
    }
}

impl AppError {
    /// Récupère les informations d'erreur formatées pour la réponse HTTP
    fn get_error_info(&self) -> (StatusCode, &'static str, String, Option<String>) {
        match self {
            // 404 Not Found
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone(), None),

            // 409 Conflict
            AppError::Duplicate(msg) => {
                (StatusCode::CONFLICT, "DUPLICATE_ENTRY", msg.clone(), None)
            }
            AppError::UserAlreadyExists => (
                StatusCode::CONFLICT,
                "USER_EXISTS",
                "Email already exists".to_string(),
                None,
            ),

            // 401 Unauthorized
            AppError::InvalidPassword => (
                StatusCode::UNAUTHORIZED,
                "INVALID_CREDENTIALS",
                "Invalid password".to_string(),
                None,
            ),
            AppError::InvalidRefreshToken => (
                StatusCode::UNAUTHORIZED,
                "INVALID_TOKEN",
                "Invalid refresh token".to_string(),
                None,
            ),
            AppError::UnauthorizedAction(msg) => {
                (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg.clone(), None)
            }

            // 400 Bad Request
            AppError::RefreshTokenExpired => (
                StatusCode::BAD_REQUEST,
                "TOKEN_EXPIRED",
                "Refresh token expired".to_string(),
                None,
            ),
            AppError::InvalidEmail => (
                StatusCode::BAD_REQUEST,
                "INVALID_EMAIL",
                "Invalid email format".to_string(),
                None,
            ),
            AppError::WeakPassword(msg) => {
                (StatusCode::BAD_REQUEST, "WEAK_PASSWORD", msg.clone(), None)
            }
            AppError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                msg.clone(),
                None,
            ),
            AppError::InvalidInput(msg) => {
                (StatusCode::BAD_REQUEST, "INVALID_INPUT", msg.clone(), None)
            }
            AppError::InvalidTokenFormat => (
                StatusCode::BAD_REQUEST,
                "INVALID_TOKEN_FORMAT",
                "Token format is invalid".to_string(),
                None,
            ),

            // 429 Too Many Requests
            AppError::TooManyAttempts(msg) => (
                StatusCode::TOO_MANY_REQUESTS,
                "TOO_MANY_ATTEMPTS",
                msg.clone(),
                None,
            ),

            // 500 Internal Server Error
            AppError::PasswordHashingFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "HASHING_ERROR",
                "An error occurred while processing your request".to_string(),
                Some(msg.clone()),
            ),
            AppError::TokenGenerationFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "TOKEN_ERROR",
                "An error occurred while generating token".to_string(),
                Some(msg.clone()),
            ),
            AppError::DatabaseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                "An error occurred with the database".to_string(),
                Some(msg.clone()),
            ),
            AppError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "An internal server error occurred".to_string(),
                Some(msg.clone()),
            ),
        }
    }

    // === Constructeurs helpers ===
    pub fn not_found(msg: impl Into<String>) -> Self {
        AppError::NotFound(msg.into())
    }

    pub fn duplicate(msg: impl Into<String>) -> Self {
        AppError::Duplicate(msg.into())
    }

    pub fn database(msg: impl Into<String>) -> Self {
        AppError::DatabaseError(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        AppError::InternalServerError(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        AppError::ValidationError(msg.into())
    }

    pub fn invalid_input(msg: impl Into<String>) -> Self {
        AppError::InvalidInput(msg.into())
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        AppError::UnauthorizedAction(msg.into())
    }

    #[expect(dead_code, reason = "Planned for rate limiting feature")]
    pub fn too_many_attempts(msg: impl Into<String>) -> Self {
        AppError::TooManyAttempts(msg.into())
    }

    pub fn hashing_failed(msg: impl Into<String>) -> Self {
        AppError::PasswordHashingFailed(msg.into())
    }

    pub fn token_generation_failed(msg: impl Into<String>) -> Self {
        AppError::TokenGenerationFailed(msg.into())
    }

    /// Retourne le code de statut HTTP
    #[expect(dead_code, reason = "Used in unit tests")]
    pub fn status_code(&self) -> StatusCode {
        self.get_error_info().0
    }
}

// === Conversions automatiques depuis d'autres types d'erreurs ===

// Depuis RepositoryError
impl From<crate::db::error::RepositoryError> for AppError {
    fn from(err: crate::db::error::RepositoryError) -> Self {
        match err {
            crate::db::error::RepositoryError::NotFound(msg) => AppError::not_found(&msg),
            crate::db::error::RepositoryError::UniqueViolation(msg) => AppError::duplicate(&msg),
            crate::db::error::RepositoryError::PoolError(msg) => AppError::database(&msg),
            crate::db::error::RepositoryError::ForeignKeyViolation(msg) => AppError::database(&msg),
            crate::db::error::RepositoryError::DatabaseError(msg) => AppError::database(&msg),
        }
    }
}

// Depuis JwtError
impl From<crate::auth::jwt::JwtError> for AppError {
    fn from(err: crate::auth::jwt::JwtError) -> Self {
        match err {
            crate::auth::jwt::JwtError::GenerationFailed(e) => {
                AppError::token_generation_failed(e.to_string())
            }
            crate::auth::jwt::JwtError::VerificationFailed(_) => AppError::InvalidRefreshToken,
        }
    }
}

// Depuis String (erreurs externes)
impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::internal(err)
    }
}

// Depuis &str
impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::internal(err.to_string())
    }
}

// Depuis serde_json::Error
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::invalid_input(format!("JSON error: {}", err))
    }
}

// Depuis uuid::Error
impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        AppError::invalid_input(format!("Invalid UUID: {}", err))
    }
}

// Depuis chrono::ParseError
impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        AppError::invalid_input(format!("Invalid date format: {}", err))
    }
}

// Depuis axum::extract::rejection::JsonRejection
impl From<axum::extract::rejection::JsonRejection> for AppError {
    fn from(err: axum::extract::rejection::JsonRejection) -> Self {
        AppError::invalid_input(format!("Invalid JSON: {}", err))
    }
}

// Depuis axum::http::uri::InvalidUri
impl From<axum::http::uri::InvalidUri> for AppError {
    fn from(err: axum::http::uri::InvalidUri) -> Self {
        AppError::invalid_input(format!("Invalid URI: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_displays_correct_message() {
        let err = AppError::not_found("User");
        assert_eq!(err.to_string(), "Not found: User");
    }

    #[test]
    fn not_found_maps_to_404_status() {
        assert_eq!(
            AppError::not_found("test").status_code(),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn invalid_password_maps_to_401_status() {
        assert_eq!(
            AppError::InvalidPassword.status_code(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn validation_error_maps_to_400_status() {
        assert_eq!(
            AppError::validation("test").status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn internal_error_maps_to_500_status() {
        assert_eq!(
            AppError::internal("test").status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn too_many_attempts_maps_to_429_status() {
        assert_eq!(
            AppError::too_many_attempts("limit exceeded").status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[test]
    fn not_found_into_response_sets_404_status() {
        let err = AppError::not_found("User");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
