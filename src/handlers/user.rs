
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::auth::services::AuthService;
use crate::dto::requests::ChangePasswordRequest;
use crate::dto::responses::UserResponse;
use crate::error::AppError;
use crate::auth::extractors::AuthClaims;
use std::sync::Arc;

/// GET /users/me
/// Récupère le profil de l'utilisateur courant
#[utoipa::path(
    get,
    path = "/users/me",
    tag = "Users",
    responses(
        (status = 200, description = "OK", body = crate::dto::responses::UserResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_current_user(
    claims: AuthClaims,
    Extension(service): Extension<Arc<AuthService>>,
) -> Result<Json<UserResponse>, AppError> {
    let user = service.get_current_user(claims.sub)?;
    Ok(Json(user))
}

/// GET /users/:id
/// Récupère un utilisateur par son ID
#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "Users",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "OK", body = crate::dto::responses::UserResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Not found", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_user_by_id(
    Path(user_id): Path<Uuid>,
    _claims: AuthClaims,
    Extension(service): Extension<Arc<AuthService>>,
) -> Result<Json<UserResponse>, AppError> {
    let user = service.get_user_by_id(user_id)?;
    Ok(Json(user))
}

/// DELETE /users/:id
/// Supprime un utilisateur
#[utoipa::path(
    delete,
    path = "/users/{id}",
    tag = "Users",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "No Content"),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse)
    )
)]
pub async fn delete_user(
    Path(user_id): Path<Uuid>,
    claims: AuthClaims,
    Extension(service): Extension<Arc<AuthService>>,
) -> Result<StatusCode, AppError> {
    // Vérifier que l'utilisateur supprime son propre compte
    if claims.sub != user_id {
        return Err(AppError::unauthorized("You can only delete your own account"));
    }

    service.delete_user(user_id)?;
    Ok(StatusCode::NO_CONTENT)
}


/// POST /users/:id/change-password
/// Change le mot de passe de l'utilisateur
#[utoipa::path(
    post,
    path = "/users/{id}/change-password",
    tag = "Users",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = crate::dto::requests::ChangePasswordRequest,
    responses(
        (status = 200, description = "OK"),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 400, description = "Validation error", body = crate::error::ErrorResponse)
    )
)]
pub async fn change_password(
    Path(user_id): Path<Uuid>,
    claims: AuthClaims,
    Extension(service): Extension<Arc<AuthService>>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
    // Vérifier que l'utilisateur change son propre password
    if claims.sub != user_id {
        return Err(AppError::unauthorized("You can only change your own password"));
    }

    service.change_password(user_id, &payload.old_password, &payload.new_password)?;
    Ok(StatusCode::OK)
}