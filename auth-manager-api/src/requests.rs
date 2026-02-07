use serde::{Deserialize, Serialize};

// -------- REQUEST DTOs --------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterRequest {
    pub email: String,
    pub username: String,
    pub password: String, // Plain text
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: String, // Plain text
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}
