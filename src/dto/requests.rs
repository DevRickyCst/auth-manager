use serde::Deserialize;

// -------- REQUEST DTOs --------
#[derive(Deserialize, Debug, Clone)]
pub struct RegisterRequest {
    pub email: String,
    pub username: String,
    pub password: String,  // Plain text
}

#[derive(Deserialize, Debug, Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,  // Plain text
}

#[derive(Deserialize, Debug, Clone)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}