use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, Duration};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
}

pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtManager {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
        }
    }

    pub fn generate_token(&self, user_id: Uuid, expires_in_hours: i64) -> Result<String, String> {
        let now = Utc::now();
        let exp = (now + Duration::hours(expires_in_hours)).timestamp();
        
        let claims = Claims {
            sub: user_id,
            exp,
            iat: now.timestamp(),
        };
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| format!("Token generation failed: {}", e))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, String> {
        decode(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|e| format!("Token verification failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::JwtManager;

    #[test]
    fn test_jwt_generate_and_verify() {
        let secret = "my_secret_key";
        let jwt_manager = JwtManager::new(secret);
        let user_id = uuid::Uuid::new_v4();
        let token = jwt_manager.generate_token(user_id, 1).expect("Token generation failed");
        let claims = jwt_manager.verify_token(&token).expect("Token verification failed");
        assert_eq!(claims.sub, user_id);
    }
}

    // ============================================
    // Helper
    // ============================================
    fn get_jwt_manager() -> JwtManager {
        JwtManager::new("my_secret_key_for_tests")
    }

    // ============================================
    // Test 1: Créer un token
    // ============================================
    #[test]
    fn test_generate_token_success() {
        // Arrange
        let jwt = get_jwt_manager();
        let user_id = Uuid::new_v4();
        let expires_in = 1;

        // Act
        let result = jwt.generate_token(user_id, expires_in);

        // Assert
        assert!(result.is_ok(), "Token generation should succeed");
        let token = result.unwrap();
        assert!(!token.is_empty(), "Token should not be empty");
        assert!(token.contains('.'), "JWT should have dots (header.payload.signature)");
    }

    // ============================================
    // Test 2: Vérifier un token valide
    // ============================================
    #[test]
    fn test_verify_token_success() {
        // Arrange
        let jwt = get_jwt_manager();
        let user_id = Uuid::new_v4();
        let token = jwt.generate_token(user_id, 1).expect("Failed to generate token");

        // Act
        let result = jwt.verify_token(&token);

        // Assert
        assert!(result.is_ok(), "Token verification should succeed");
        let claims = result.unwrap();
        assert_eq!(claims.sub, user_id, "User ID should match");
        assert!(claims.exp > claims.iat, "Expiry should be after issued time");
    }

        // ============================================
    // Test 3: Token invalide
    // ============================================
    #[test]
    fn test_verify_invalid_token() {
        // Arrange
        let jwt = get_jwt_manager();
        let invalid_token = "invalid.token.here";

        // Act
        let result = jwt.verify_token(invalid_token);

        // Assert
        assert!(result.is_err(), "Invalid token should fail verification");
        assert!(result.unwrap_err().contains("Token verification failed"));
    }