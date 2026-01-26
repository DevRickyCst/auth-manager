// src/auth/services.rs

use crate::dto::responses::{UserResponse, LoginResponse, RefreshTokenResponse};
use crate::dto::requests::{RegisterRequest, LoginRequest, RefreshTokenRequest};
use crate::error::AppError;

use crate::db::models::user::NewUser;
use crate::db::models::refresh_token::NewRefreshToken;

use crate::db::repositories::user_repository::UserRepository;
use crate::db::repositories::refresh_token_repository::RefreshTokenRepository;
use crate::db::repositories::login_attempt_repository::LoginAttemptRepository;

use chrono::Utc;

pub struct AuthService {
    jwt_manager: super::jwt::JwtManager,
}

impl AuthService {
    pub fn new(jwt_manager: super::jwt::JwtManager) -> Self {
        Self { jwt_manager }
    }

    /// Inscription d'un nouvel utilisateur
    pub fn register(register_request: RegisterRequest) -> Result<UserResponse, AppError> {
        // Validation email
        if !Self::is_valid_email(&register_request.email) {
            return Err(AppError::InvalidEmail);
        }

        // Validation password
        if !Self::is_strong_password(&register_request.password) {
            return Err(AppError::WeakPassword(
                "Password must be at least 8 characters with uppercase, lowercase and numbers"
                    .to_string(),
            ));
        }

        // Vérifier que l'email n'existe pas
        let user = UserRepository::find_by_email(&register_request.email)?;
        if user.is_some() {
            return Err(AppError::UserAlreadyExists);
        }

        // Hash le password
        let password_hash = super::password::PasswordManager::hash(&register_request.password)
            .map_err(|e| AppError::hashing_failed(e))?;

        let new_user = NewUser {
            email: register_request.email.to_string(),
            username: register_request.username.to_string(),
            password_hash: Some(password_hash),
        };

        // Crée l'utilisateur
        UserRepository::create(&new_user)
            .map(|user| user.into())
            .map_err(AppError::from)
    }

    /// Connexion d'un utilisateur
    pub fn login(
        &self,
        login_request: LoginRequest,
        user_agent: Option<String>,
    ) -> Result<LoginResponse, AppError> {
        // Valide l'email
        if !Self::is_valid_email(&login_request.email) {
            return Err(AppError::InvalidEmail);
        }

        // Recherche l'utilisateur
        let user = match UserRepository::find_by_email(&login_request.email) {
            Ok(Some(u)) => u,
            Ok(None) => {
                // Enregistre la tentative échouée
                let _ = LoginAttemptRepository::create(None, false, user_agent);
                return Err(AppError::not_found("User"));
            }
            Err(e) => return Err(AppError::from(e)),
        };

        // Vérifie le password
        let password_hash = user.password_hash.as_ref().ok_or_else(|| {
            AppError::database("Password not set for user")
        })?;

        if !super::password::PasswordManager::verify(&login_request.password, password_hash)
            .map_err(|e| AppError::hashing_failed(e))?
        {
            // Enregistre la tentative échouée
            let _ = LoginAttemptRepository::create(Some(user.id), false, user_agent.clone());
            return Err(AppError::InvalidPassword);
        }

        // Génère les tokens
        let access_token = self
            .jwt_manager
            .generate_token(user.id, 1)
            .map_err(|e| AppError::token_generation_failed(e))?;

        let refresh_token = uuid::Uuid::new_v4().to_string();
        let refresh_token_hash = super::password::PasswordManager::hash(&refresh_token)
            .map_err(|e| AppError::hashing_failed(e))?;

        let new_refresh_token = NewRefreshToken {
            user_id: user.id,
            token_hash: refresh_token_hash,
            expires_at: Utc::now() + chrono::Duration::days(7),
        };

        // Enregistre le refresh token
        RefreshTokenRepository::create(&new_refresh_token)?;

        // Met à jour le last_login
        UserRepository::update_last_login(user.id)?;

        // Enregistre la tentative réussie
        let _ = LoginAttemptRepository::create(Some(user.id), true, user_agent);

        Ok(LoginResponse {
            access_token,
            refresh_token,
            user: user.into(),
            expires_in: 3600,
        })
    }

    /// Rafraîchit les tokens
    pub fn refresh_token(
        &self,
        refresh_token_request: RefreshTokenRequest,
    ) -> Result<RefreshTokenResponse, AppError> {
        if refresh_token_request.refresh_token.is_empty() {
            return Err(AppError::InvalidRefreshToken);
        }

        // Recherche le refresh token
        let old_token = RefreshTokenRepository::find_by_hash(&refresh_token_request.refresh_token)
            .map_err(AppError::from)?
            .ok_or(AppError::InvalidRefreshToken)?;

        // Vérifie qu'il n'a pas expiré
        if old_token.expires_at < Utc::now() {
            return Err(AppError::RefreshTokenExpired);
        }

        // Génère un nouveau access token
        let access_token = self
            .jwt_manager
            .generate_token(old_token.user_id, 1)
            .map_err(|e| AppError::token_generation_failed(e))?;

        // Supprime l'ancien refresh token
        let _ = RefreshTokenRepository::delete(old_token.id);

        // Crée un nouveau refresh token
        let new_refresh_token_str = uuid::Uuid::new_v4().to_string();
        let new_refresh_token_hash =
            super::password::PasswordManager::hash(&new_refresh_token_str)
                .map_err(|e| AppError::hashing_failed(e))?;

        let new_refresh_token = NewRefreshToken {
            user_id: old_token.user_id,
            token_hash: new_refresh_token_hash,
            expires_at: Utc::now() + chrono::Duration::days(7),
        };

        RefreshTokenRepository::create(&new_refresh_token)?;

        Ok(RefreshTokenResponse {
            access_token,
            expires_in: 3600,
        })
    }

    // === Helpers de validation ===

    fn is_valid_email(email: &str) -> bool {
        email.contains('@') && email.contains('.') && email.len() > 5
    }

    fn is_strong_password(password: &str) -> bool {
        password.len() >= 8
            && password.chars().any(|c| c.is_uppercase())
            && password.chars().any(|c| c.is_lowercase())
            && password.chars().any(|c| c.is_numeric())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_register_request() -> RegisterRequest {
        RegisterRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            password: "TestPassword123!".to_string(),
        }
    }

    #[test]
    fn test_register_success() {
        let register_request = create_test_register_request();
        let user = AuthService::register(register_request).expect("Registration should succeed");

        let result = UserRepository::find_by_email("test@example.com");
        assert!(result.is_ok(), "Should find the newly registered user");

        let _ = UserRepository::delete(user.id);
    }

    #[test]
    fn test_register_invalid_email() {
        let register_request = RegisterRequest {
            email: "invalid-email".to_string(),
            username: "testuser".to_string(),
            password: "TestPassword123!".to_string(),
        };
        
        let result = AuthService::register(register_request);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_weak_password() {
        let register_request = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            password: "weak".to_string(),
        };
        
        let result = AuthService::register(register_request);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_duplicate_email() {
        let register_request = create_test_register_request();
        
        // Première inscription
        let _ = AuthService::register(register_request.clone())
            .expect("First registration should succeed");

        // Deuxième inscription avec le même email
        let result = AuthService::register(register_request);
        assert!(result.is_err());
    }

    #[test]
    fn test_login_success() {
        let register_request = create_test_register_request();
        let _ = AuthService::register(register_request).expect("Registration should succeed");

        let jwt_manager = crate::auth::jwt::JwtManager::new("secret_key");
        let auth_service = AuthService::new(jwt_manager);

        let login_request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "TestPassword123!".to_string(),
        };

        let login_response =
            auth_service.login(login_request, None).expect("Login should succeed");
        assert_eq!(login_response.user.email, "test@example.com");

        let _ = UserRepository::delete(login_response.user.id);
    }

    #[test]
    fn test_login_invalid_password() {
        let register_request = create_test_register_request();
        let user = AuthService::register(register_request).expect("Registration should succeed");

        let jwt_manager = crate::auth::jwt::JwtManager::new("secret_key");
        let auth_service = AuthService::new(jwt_manager);

        let login_request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "WrongPassword123!".to_string(),
        };

        let result = auth_service.login(login_request, None);
        assert!(result.is_err());

        let _ = UserRepository::delete(user.id);
    }

    #[test]
    fn test_login_user_not_found() {
        let jwt_manager = crate::auth::jwt::JwtManager::new("secret_key");
        let auth_service = AuthService::new(jwt_manager);

        let login_request = LoginRequest {
            email: "nonexistent@example.com".to_string(),
            password: "TestPassword123!".to_string(),
        };

        let result = auth_service.login(login_request, None);
        assert!(result.is_err());
    }
}