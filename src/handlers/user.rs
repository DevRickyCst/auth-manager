use axum::{
    Json,
    extract::{Extension, Path},
};
use uuid::Uuid;

use crate::auth::extractors::AuthClaims;
use crate::auth::services::AuthService;
use crate::error::AppError;
use crate::response::AppResponse;
use auth_manager_api::{ChangePasswordRequest, UserResponse};
use std::sync::Arc;

/// GET /users/me
/// Récupère le profil de l'utilisateur courant
pub async fn get_current_user(
    claims: AuthClaims,
    Extension(service): Extension<Arc<AuthService>>,
) -> Result<AppResponse<UserResponse>, AppError> {
    let user = service.get_current_user(claims.sub)?;
    Ok(AppResponse::ok(user))
}

/// GET /users/:id
/// Récupère un utilisateur par son ID
pub async fn get_user_by_id(
    Path(user_id): Path<Uuid>,
    _claims: AuthClaims,
    Extension(service): Extension<Arc<AuthService>>,
) -> Result<AppResponse<UserResponse>, AppError> {
    let user = service.get_user_by_id(user_id)?;
    Ok(AppResponse::ok(user))
}

/// DELETE /users/:id
/// Supprime un utilisateur
pub async fn delete_user(
    Path(user_id): Path<Uuid>,
    claims: AuthClaims,
    Extension(service): Extension<Arc<AuthService>>,
) -> Result<AppResponse<()>, AppError> {
    // Vérifier que l'utilisateur supprime son propre compte
    if claims.sub != user_id {
        return Err(AppError::unauthorized(
            "You can only delete your own account",
        ));
    }

    service.delete_user(user_id)?;
    Ok(AppResponse::no_content())
}

/// POST /users/:id/change-password
/// Change le mot de passe de l'utilisateur
pub async fn change_password(
    Path(user_id): Path<Uuid>,
    claims: AuthClaims,
    Extension(service): Extension<Arc<AuthService>>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<AppResponse<serde_json::Value>, AppError> {
    // Vérifier que l'utilisateur change son propre password
    if claims.sub != user_id {
        return Err(AppError::unauthorized(
            "You can only change your own password",
        ));
    }

    service.change_password(user_id, &payload.old_password, &payload.new_password)?;
    Ok(AppResponse::ok(serde_json::json!({
        "message": "Password changed successfully"
    })))
}
