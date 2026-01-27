use serde::Deserialize;
use utoipa::ToSchema;

// -------- REQUEST DTOs --------
#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub username: String,
    pub password: String,  // Plain text
}

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,  // Plain text
}

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}