// src/error.rs

use auth_manager_api::ErrorResponse;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::fmt;

#[derive(Debug, Clone)]
pub enum AppError {
    // === Erreurs Repository ===
    NotFound(String),
    Duplicate(String),
    DatabaseError(String),

    // === Erreurs d'Authentification ===
    InvalidPassword,
    UserAlreadyExists,
    InvalidRefreshToken,
    RefreshTokenExpired,
    InvalidEmail,
    WeakPassword(String),

    // === Erreurs de Hashing/Cryptographie ===
    PasswordHashingFailed(String),
    TokenGenerationFailed(String),
    InvalidTokenFormat,

    // === Erreurs de Validation ===
    ValidationError(String),
    #[allow(dead_code)]
    MissingField(String),
    InvalidInput(String),

    // === Erreurs métier ===
    UnauthorizedAction(String),
    #[allow(dead_code)]
    ResourceLocked(String),
    #[allow(dead_code)]
    TooManyAttempts(String),

    // === Erreurs internes ===
    InternalServerError(String),
    #[allow(dead_code)]
    ConfigurationError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Repository
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Duplicate(msg) => write!(f, "Already exists: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),

            // Auth
            AppError::InvalidPassword => write!(f, "Invalid password"),
            AppError::UserAlreadyExists => write!(f, "Email already exists"),
            AppError::InvalidRefreshToken => write!(f, "Invalid refresh token"),
            AppError::RefreshTokenExpired => write!(f, "Refresh token expired"),
            AppError::InvalidEmail => write!(f, "Invalid email format"),
            AppError::WeakPassword(msg) => write!(f, "Password too weak: {}", msg),

            // Crypto
            AppError::PasswordHashingFailed(msg) => write!(f, "Password hashing failed: {}", msg),
            AppError::TokenGenerationFailed(msg) => write!(f, "Token generation failed: {}", msg),
            AppError::InvalidTokenFormat => write!(f, "Invalid token format"),

            // Validation
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::MissingField(field) => write!(f, "Missing required field: {}", field),
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),

            // Business
            AppError::UnauthorizedAction(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::ResourceLocked(msg) => write!(f, "Resource locked: {}", msg),
            AppError::TooManyAttempts(msg) => write!(f, "Too many attempts: {}", msg),

            // Internal
            AppError::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
            AppError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message, details) = self.get_error_info();

        let body = Json(ErrorResponse {
            error: error_code.to_string(),
            message,
            details,
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
            AppError::MissingField(field) => (
                StatusCode::BAD_REQUEST,
                "MISSING_FIELD",
                format!("Missing required field: {}", field),
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

            // 423 Locked
            AppError::ResourceLocked(msg) => {
                (StatusCode::LOCKED, "RESOURCE_LOCKED", msg.clone(), None)
            }

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
            AppError::ConfigurationError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "CONFIG_ERROR",
                "Server configuration error".to_string(),
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

    #[allow(dead_code)]
    pub fn missing_field(field: impl Into<String>) -> Self {
        AppError::MissingField(field.into())
    }

    pub fn invalid_input(msg: impl Into<String>) -> Self {
        AppError::InvalidInput(msg.into())
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        AppError::UnauthorizedAction(msg.into())
    }

    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    fn test_error_display() {
        let err = AppError::not_found("User");
        assert_eq!(err.to_string(), "Not found: User");
    }

    #[test]
    fn test_not_found_status() {
        assert_eq!(
            AppError::not_found("test").status_code(),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn test_unauthorized_status() {
        assert_eq!(
            AppError::InvalidPassword.status_code(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn test_validation_status() {
        assert_eq!(
            AppError::validation("test").status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_internal_status() {
        assert_eq!(
            AppError::internal("test").status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_too_many_attempts_status() {
        assert_eq!(
            AppError::too_many_attempts("limit exceeded").status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[test]
    fn test_error_response() {
        let err = AppError::not_found("User");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
