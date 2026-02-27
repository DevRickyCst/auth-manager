use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("Token generation failed: {0}")]
    GenerationFailed(jsonwebtoken::errors::Error),
    #[error("Token verification failed: {0}")]
    VerificationFailed(jsonwebtoken::errors::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Clone)]
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiration_hours: i64,
}

impl JwtManager {
    pub fn new(secret: &str, expiration_hours: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
            expiration_hours,
        }
    }

    /// Génère un access token avec la durée configurée
    pub fn generate_access_token(&self, user_id: Uuid) -> Result<String, JwtError> {
        self.generate_token(user_id, self.expiration_hours)
    }

    pub fn expiration_hours(&self) -> i64 {
        self.expiration_hours
    }

    pub fn generate_token(&self, user_id: Uuid, expires_in_hours: i64) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = (now + Duration::hours(expires_in_hours)).timestamp();

        let claims = Claims {
            sub: user_id,
            exp,
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(JwtError::GenerationFailed)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, JwtError> {
        decode(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(JwtError::VerificationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::{JwtError, JwtManager, Uuid};

    fn make_jwt_manager() -> JwtManager {
        JwtManager::new("my_secret_key_for_tests", 1)
    }

    #[test]
    fn generate_and_verify_succeeds_with_valid_token() {
        let jwt_manager = JwtManager::new("my_secret_key", 1);
        let user_id = Uuid::new_v4();
        let token = jwt_manager
            .generate_token(user_id, 1)
            .expect("Token generation failed");
        let claims = jwt_manager
            .verify_token(&token)
            .expect("Token verification failed");
        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn generate_token_returns_jwt_with_correct_format() {
        let jwt = make_jwt_manager();
        let user_id = Uuid::new_v4();

        let token = jwt
            .generate_token(user_id, 1)
            .expect("Token generation should succeed");

        assert!(!token.is_empty(), "Token should not be empty");
        assert!(token.contains('.'), "JWT should have dots (header.payload.signature)");
    }

    #[test]
    fn verify_token_returns_correct_claims() {
        let jwt = make_jwt_manager();
        let user_id = Uuid::new_v4();
        let token = jwt
            .generate_token(user_id, 1)
            .expect("Failed to generate token");

        let claims = jwt.verify_token(&token).expect("Token verification should succeed");

        assert_eq!(claims.sub, user_id, "User ID should match");
        assert!(claims.exp > claims.iat, "Expiry should be after issued time");
    }

    #[test]
    fn verify_token_fails_with_invalid_input() {
        let jwt = make_jwt_manager();

        let result = jwt.verify_token("invalid.token.here");

        assert!(matches!(result.unwrap_err(), JwtError::VerificationFailed(_)));
    }
}
