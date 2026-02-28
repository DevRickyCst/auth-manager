// src/auth/services.rs

use crate::error::AppError;
use auth_manager_api::{
    LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse, RegisterRequest,
    UserResponse,
};

use crate::db::models::refresh_token::NewRefreshToken;
use crate::db::models::user::NewUser;

use crate::db::repositories::login_attempt_repository::LoginAttemptRepository;
use crate::db::repositories::refresh_token_repository::RefreshTokenRepository;
use crate::db::repositories::user_repository::UserRepository;

use chrono::Utc;

const MAX_FAILED_ATTEMPTS: i64 = 5;
const LOCKOUT_WINDOW_MINUTES: i64 = 15;

pub struct AuthService {
    jwt_manager: super::jwt::JwtManager,
}

impl AuthService {
    pub fn new(jwt_manager: super::jwt::JwtManager) -> Self {
        Self { jwt_manager }
    }

    /// Returns the current authenticated user's profile.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotFound`] if the user no longer exists, or a database error.
    pub fn get_current_user(user_id: uuid::Uuid) -> Result<UserResponse, AppError> {
        Self::get_user_by_id(user_id)
    }

    /// Revokes all refresh tokens for the given user (logout).
    ///
    /// # Errors
    ///
    /// Returns a database error if token deletion fails.
    pub fn logout(user_id: uuid::Uuid) -> Result<(), AppError> {
        RefreshTokenRepository::delete_by_user(user_id).map_err(AppError::from)?;
        Ok(())
    }

    /// Fetches a user by their ID and maps to a public response DTO.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotFound`] if no user with that ID exists, or a database error.
    pub fn get_user_by_id(user_id: uuid::Uuid) -> Result<UserResponse, AppError> {
        UserRepository::find_by_id(user_id)
            .map_err(AppError::from)?
            .map(UserResponse::from)
            .ok_or_else(|| AppError::not_found("User not found"))
    }

    /// Permanently deletes a user account.
    ///
    /// # Errors
    ///
    /// Returns a database error if deletion fails.
    pub fn delete_user(user_id: uuid::Uuid) -> Result<(), AppError> {
        UserRepository::delete(user_id)?;
        Ok(())
    }

    /// Changes the user's password after verifying the current one.
    ///
    /// # Errors
    ///
    /// - [`AppError::WeakPassword`] if `new_password` does not meet strength requirements.
    /// - [`AppError::NotFound`] if the user does not exist.
    /// - [`AppError::InvalidPassword`] if `old_password` does not match the stored hash.
    /// - [`AppError::DatabaseError`] on persistence failures.
    pub fn change_password(
        user_id: uuid::Uuid,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        if !Self::is_strong_password(new_password) {
            return Err(AppError::WeakPassword(
                "Password must be at least 8 characters with uppercase, lowercase and numbers"
                    .to_string(),
            ));
        }

        let user = UserRepository::find_by_id(user_id)?;
        let user = user.ok_or_else(|| AppError::not_found("User not found"))?;

        let password_hash = user
            .password_hash
            .as_ref()
            .ok_or_else(|| AppError::database("Password not set for user"))?;

        if !super::password::PasswordManager::verify(old_password, password_hash)
            .map_err(AppError::from)?
        {
            return Err(AppError::InvalidPassword);
        }

        let new_password_hash =
            super::password::PasswordManager::hash(new_password).map_err(AppError::from)?;

        UserRepository::update_password(user_id, &new_password_hash)?;
        Ok(())
    }

    /// Registers a new user account.
    ///
    /// # Errors
    ///
    /// - [`AppError::InvalidEmail`] if the email format is invalid.
    /// - [`AppError::WeakPassword`] if the password does not meet strength requirements.
    /// - [`AppError::UserAlreadyExists`] if the email is already registered.
    /// - [`AppError::DatabaseError`] on persistence failures.
    pub fn register(register_request: RegisterRequest) -> Result<UserResponse, AppError> {
        if !Self::is_valid_email(&register_request.email) {
            return Err(AppError::InvalidEmail);
        }

        if !Self::is_strong_password(&register_request.password) {
            return Err(AppError::WeakPassword(
                "Password must be at least 8 characters with uppercase, lowercase and numbers"
                    .to_string(),
            ));
        }

        let user = UserRepository::find_by_email(&register_request.email)?;
        if user.is_some() {
            return Err(AppError::UserAlreadyExists);
        }

        let password_hash = super::password::PasswordManager::hash(&register_request.password)
            .map_err(AppError::from)?;

        let new_user = NewUser {
            email: register_request.email,
            username: register_request.username,
            password_hash: Some(password_hash),
        };

        UserRepository::create(&new_user)
            .map(std::convert::Into::into)
            .map_err(AppError::from)
    }

    /// Authenticates a user and returns an access token + refresh token hash.
    ///
    /// The second element of the returned tuple is the **bcrypt hash** of the refresh token,
    /// intended to be stored in an `HttpOnly` cookie — never returned in the response body.
    ///
    /// # Errors
    ///
    /// - [`AppError::InvalidEmail`] if the email format is invalid.
    /// - [`AppError::NotFound`] if no user with that email exists.
    /// - [`AppError::TooManyAttempts`] if the account is temporarily locked.
    /// - [`AppError::InvalidPassword`] if the password does not match.
    /// - [`AppError::DatabaseError`] on persistence failures.
    pub fn login(
        &self,
        login_request: &LoginRequest,
        user_agent: Option<String>,
    ) -> Result<(LoginResponse, String), AppError> {
        if !Self::is_valid_email(&login_request.email) {
            return Err(AppError::InvalidEmail);
        }

        let user = match UserRepository::find_by_email(&login_request.email) {
            Ok(Some(u)) => u,
            Ok(None) => {
                let _ = LoginAttemptRepository::create(None, false, user_agent)
                    .inspect_err(|e| tracing::warn!("Failed to log login attempt: {e}"));
                return Err(AppError::not_found("User"));
            }
            Err(e) => return Err(AppError::from(e)),
        };

        let failed_count =
            LoginAttemptRepository::count_failed_attempts(user.id, LOCKOUT_WINDOW_MINUTES)
                .map_err(AppError::from)?;
        if failed_count >= MAX_FAILED_ATTEMPTS {
            return Err(AppError::TooManyAttempts(format!(
                "Account temporarily locked after {MAX_FAILED_ATTEMPTS} failed attempts. Try again in {LOCKOUT_WINDOW_MINUTES} minutes."
            )));
        }

        let password_hash = user
            .password_hash
            .as_ref()
            .ok_or_else(|| AppError::database("Password not set for user"))?;

        if !super::password::PasswordManager::verify(&login_request.password, password_hash)
            .map_err(AppError::from)?
        {
            let _ = LoginAttemptRepository::create(Some(user.id), false, user_agent)
                .inspect_err(|e| tracing::warn!("Failed to log failed login attempt: {e}"));
            return Err(AppError::InvalidPassword);
        }

        let access_token = self
            .jwt_manager
            .generate_access_token(user.id)
            .map_err(AppError::from)?;

        let refresh_token = uuid::Uuid::new_v4().to_string();
        let refresh_token_hash =
            super::password::PasswordManager::hash(&refresh_token).map_err(AppError::from)?;

        let new_refresh_token = NewRefreshToken {
            user_id: user.id,
            token_hash: refresh_token_hash.clone(),
            expires_at: Utc::now() + chrono::Duration::days(7),
        };

        let _created = RefreshTokenRepository::create(&new_refresh_token)?;
        UserRepository::update_last_login(user.id)?;

        let _ = LoginAttemptRepository::create(Some(user.id), true, user_agent)
            .inspect_err(|e| tracing::warn!("Failed to log successful login attempt: {e}"));

        let resp = LoginResponse {
            access_token,
            refresh_token,
            user: user.into(),
            expires_in: self.jwt_manager.expiration_hours() * 3600,
        };

        Ok((resp, refresh_token_hash))
    }

    /// Rotates a refresh token: invalidates the old one and issues a new pair.
    ///
    /// The second element of the returned tuple is the **bcrypt hash** of the new refresh token,
    /// intended to be stored in an `HttpOnly` cookie.
    ///
    /// # Errors
    ///
    /// - [`AppError::InvalidRefreshToken`] if the token is empty or not found in the database.
    /// - [`AppError::RefreshTokenExpired`] if the token has passed its expiry date.
    /// - [`AppError::DatabaseError`] on persistence failures.
    pub fn refresh_token(
        &self,
        refresh_token_request: &RefreshTokenRequest,
    ) -> Result<(RefreshTokenResponse, String), AppError> {
        if refresh_token_request.refresh_token.is_empty() {
            return Err(AppError::InvalidRefreshToken);
        }

        let old_token = RefreshTokenRepository::find_by_hash(&refresh_token_request.refresh_token)
            .map_err(AppError::from)?
            .ok_or(AppError::InvalidRefreshToken)?;

        if old_token.expires_at < Utc::now() {
            return Err(AppError::RefreshTokenExpired);
        }

        let access_token = self
            .jwt_manager
            .generate_access_token(old_token.user_id)
            .map_err(AppError::from)?;

        RefreshTokenRepository::delete(old_token.id)
            .inspect_err(|e| {
                tracing::error!("Failed to delete old refresh token {}: {e}", old_token.id);
            })
            .ok();

        let new_refresh_token_str = uuid::Uuid::new_v4().to_string();
        let new_refresh_token_hash = super::password::PasswordManager::hash(&new_refresh_token_str)
            .map_err(AppError::from)?;

        let new_refresh_token = NewRefreshToken {
            user_id: old_token.user_id,
            token_hash: new_refresh_token_hash.clone(),
            expires_at: Utc::now() + chrono::Duration::days(7),
        };

        RefreshTokenRepository::create(&new_refresh_token)?;

        Ok((
            RefreshTokenResponse {
                access_token,
                expires_in: self.jwt_manager.expiration_hours() * 3600,
            },
            new_refresh_token_hash,
        ))
    }

    // === Helpers de validation ===

    fn is_valid_email(email: &str) -> bool {
        email.contains('@') && email.contains('.') && email.len() > 5
    }

    fn is_strong_password(password: &str) -> bool {
        if password.len() < 8 {
            return false;
        }
        let (mut upper, mut lower, mut digit) = (false, false, false);
        for c in password.chars() {
            upper |= c.is_uppercase();
            lower |= c.is_lowercase();
            digit |= c.is_ascii_digit();
            if upper && lower && digit {
                return true;
            }
        }
        upper && lower && digit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::password::PasswordManager;
    use crate::db::connection::init_test_pool;
    use crate::db::models::user::NewUser;
    use crate::db::repositories::user_repository::UserRepository;

    fn create_test_register_request() -> RegisterRequest {
        init_test_pool();

        let unique = uuid::Uuid::new_v4();
        RegisterRequest {
            email: format!("test+{unique}@example.com"),
            username: format!("testuser_{unique}"),
            password: "TestPassword123!".to_string(),
        }
    }

    #[test]
    fn register_succeeds_with_valid_data() {
        let register_request = create_test_register_request();
        let email = register_request.email.clone();
        let user = AuthService::register(register_request).expect("Registration should succeed");

        let result = UserRepository::find_by_email(&email);
        assert!(result.is_ok(), "Should find the newly registered user");

        let _ = UserRepository::delete(user.id);
    }

    #[test]
    fn register_fails_when_email_is_invalid() {
        let register_request = RegisterRequest {
            email: "invalid-email".to_string(),
            username: "testuser".to_string(),
            password: "TestPassword123!".to_string(),
        };

        let result: Result<UserResponse, AppError> = AuthService::register(register_request);
        assert!(result.is_err());
    }

    #[test]
    fn register_fails_when_password_is_weak() {
        let register_request = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            password: "weak".to_string(),
        };

        let result = AuthService::register(register_request);
        assert!(result.is_err());
    }

    #[test]
    fn register_fails_when_email_already_exists() {
        let register_request = create_test_register_request();

        // Première inscription
        let result1 = AuthService::register(register_request.clone())
            .expect("First registration should succeed");

        // Deuxième inscription avec le même email
        let result2 = AuthService::register(register_request);
        assert!(result2.is_err());

        let _ = UserRepository::delete(result1.id);
    }

    #[test]
    fn login_succeeds_with_valid_credentials() {
        let register_request = create_test_register_request();

        let email = register_request.email.clone();
        let password = register_request.password.clone();

        AuthService::register(register_request).expect("Registration should succeed");

        let jwt_manager = crate::auth::jwt::JwtManager::new("secret_key", 1);
        let auth_service = AuthService::new(jwt_manager);

        let login_request = LoginRequest {
            email: email.clone(),
            password,
        };

        let (login_response, _refresh_hash) = auth_service
            .login(&login_request, None)
            .expect("Login should succeed");

        assert_eq!(login_response.user.email, email);

        let _ = UserRepository::delete(login_response.user.id);
    }

    #[test]
    fn login_fails_with_wrong_password() {
        let register_request = create_test_register_request();
        let email = register_request.email.clone();
        let user = AuthService::register(register_request).expect("Registration should succeed");

        let jwt_manager = crate::auth::jwt::JwtManager::new("default_secret", 1);
        let auth_service = AuthService::new(jwt_manager);

        let login_request = LoginRequest {
            email,
            password: "WrongPassword123!".to_string(),
        };

        let result = auth_service.login(&login_request, None);
        assert!(result.is_err());

        let _ = UserRepository::delete(user.id);
    }

    #[test]
    fn login_fails_when_user_not_found() {
        init_test_pool();
        let jwt_manager = crate::auth::jwt::JwtManager::new("secret_key", 1);
        let auth_service = AuthService::new(jwt_manager);

        let login_request = LoginRequest {
            email: "nonexistent@example.com".to_string(),
            password: "TestPassword123!".to_string(),
        };

        let result = auth_service.login(&login_request, None);
        assert!(result.is_err());
    }

    #[test]
    fn change_password_succeeds_with_correct_old_password() {
        init_test_pool();
        // Create user with known password
        let old_hash = PasswordManager::hash("OldPass123!").expect("hash");
        let new_user = NewUser {
            email: format!("change_pw_{}@example.com", uuid::Uuid::new_v4()),
            username: "change_pw_user".to_string(),
            password_hash: Some(old_hash),
        };
        let user = UserRepository::create(&new_user).expect("create user");

        // Change password via service
        let result = AuthService::change_password(user.id, "OldPass123!", "NewPass456!");
        assert!(result.is_ok(), "Change password should succeed");

        // Verify new password
        let updated = UserRepository::find_by_id(user.id)
            .expect("find")
            .expect("exists");
        let hash = updated.password_hash.as_ref().expect("hash");
        let ok = PasswordManager::verify("NewPass456!", hash).expect("verify");
        assert!(ok);

        let _ = UserRepository::delete(user.id);
    }

    #[test]
    fn change_password_fails_when_old_password_is_wrong() {
        init_test_pool();
        let old_hash = PasswordManager::hash("OldPass123!").expect("hash");
        let new_user = NewUser {
            email: format!("change_pw_wrong_{}@example.com", uuid::Uuid::new_v4()),
            username: "change_pw_wrong_user".to_string(),
            password_hash: Some(old_hash),
        };
        let user = UserRepository::create(&new_user).expect("create user");

        let result = AuthService::change_password(user.id, "WrongOld!", "NewPass456!");
        assert!(result.is_err(), "Should fail with invalid old password");

        let _ = UserRepository::delete(user.id);
    }
}
