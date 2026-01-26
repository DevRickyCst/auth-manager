use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub email_verified: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Debug)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
    pub expires_in: i64,
}

#[derive(Serialize, Debug)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}
