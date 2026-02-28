use axum::{Json, extract::Path};
use uuid::Uuid;

use crate::auth::extractors::AuthClaims;
use crate::auth::services::AuthService;
use crate::error::AppError;
use crate::response::AppResponse;
use auth_manager_api::{ChangePasswordRequest, UserResponse};

/// GET /users/me
/// Récupère le profil de l'utilisateur courant
pub async fn get_current_user(claims: AuthClaims) -> Result<AppResponse<UserResponse>, AppError> {
    let user = AuthService::get_current_user(claims.sub)?;
    Ok(AppResponse::ok(user))
}

/// GET /users/:id
/// Récupère un utilisateur par son ID
pub async fn get_user_by_id(
    Path(user_id): Path<Uuid>,
    _claims: AuthClaims,
) -> Result<AppResponse<UserResponse>, AppError> {
    let user = AuthService::get_user_by_id(user_id)?;
    Ok(AppResponse::ok(user))
}

/// DELETE /users/:id
/// Supprime un utilisateur
pub async fn delete_user(
    Path(user_id): Path<Uuid>,
    claims: AuthClaims,
) -> Result<AppResponse<()>, AppError> {
    // Vérifier que l'utilisateur supprime son propre compte
    if claims.sub != user_id {
        return Err(AppError::unauthorized(
            "You can only delete your own account",
        ));
    }

    AuthService::delete_user(user_id)?;
    Ok(AppResponse::no_content())
}

/// POST /users/:id/change-password
/// Change le mot de passe de l'utilisateur
pub async fn change_password(
    Path(user_id): Path<Uuid>,
    claims: AuthClaims,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<AppResponse<serde_json::Value>, AppError> {
    // Vérifier que l'utilisateur change son propre password
    if claims.sub != user_id {
        return Err(AppError::unauthorized(
            "You can only change your own password",
        ));
    }

    AuthService::change_password(user_id, &payload.old_password, &payload.new_password)?;
    Ok(AppResponse::ok(serde_json::json!({
        "message": "Password changed successfully"
    })))
}
