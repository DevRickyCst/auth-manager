use crate::db::models::refresh_token::{RefreshToken, NewRefreshToken};
use crate::db::schema::refresh_tokens;
use diesel::prelude::*;
use uuid::Uuid;
use crate::db::connection::get_connection;
use chrono::Utc;
use crate::db::error::{map_diesel_error, RepositoryError};
pub struct RefreshTokenRepository;

impl RefreshTokenRepository {


    pub fn create(new_refresh_token: &NewRefreshToken
    ) -> Result<RefreshToken, RepositoryError> {
        
        let mut conn = get_connection()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        
        
        diesel::insert_into(refresh_tokens::table)
            .values(new_refresh_token)
            .get_result::<RefreshToken>(&mut conn)
            .map_err(map_diesel_error)
        }

    pub fn find_by_hash(hash: &str) -> Result<Option<RefreshToken>, RepositoryError> {
        let hash = hash.to_string();
        let mut conn = get_connection()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        refresh_tokens::table
            .filter(refresh_tokens::token_hash.eq(hash))
            .filter(refresh_tokens::expires_at.gt(Utc::now()))
            .first::<RefreshToken>(&mut conn)
            .optional()
            .map_err(map_diesel_error)
    }


    pub fn delete(id: Uuid) -> Result<(), RepositoryError> {
        let mut conn = get_connection()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        diesel::delete(refresh_tokens::table.filter(refresh_tokens::id.eq(id)))
            .execute(&mut conn)
            .map_err(map_diesel_error)?;
        
        Ok(())
    }

    pub fn delete_by_user(user_id: Uuid) -> Result<(), RepositoryError> {
        let mut conn = get_connection()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        diesel::delete(refresh_tokens::table.filter(refresh_tokens::user_id.eq(user_id)))
            .execute(&mut conn)
            .map_err(map_diesel_error)?;
        
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::user::NewUser;
    use crate::db::repositories::user_repository::UserRepository;

    fn create_test_user() -> Uuid {
        let new_user = NewUser {
            email: format!("test_{}@example.com", Uuid::new_v4()),
            username: format!("testuser_{}", Uuid::new_v4()),
            password_hash: Some("test_hash".to_string()),
        };

        let user = UserRepository::create(&new_user)
            .expect("Failed to create test user");
        user.id
    }

    fn create_test_refresh_token(user_id: Uuid) -> NewRefreshToken {
        NewRefreshToken {
            user_id,
            token_hash: format!("test_hash_{}", Uuid::new_v4()),
            expires_at: Utc::now() + chrono::Duration::days(7),
        }
    }

    // ============================================
    // Test 1: Créer un refresh token
    // ============================================
    #[test]
    fn test_create_refresh_token_success() {
        // Arrange
        let user_id = create_test_user();  // ← Créer le user d'abord
        let new_token = create_test_refresh_token(user_id);

        // Act
        let result = RefreshTokenRepository::create(&new_token);

        // Assert
        assert!(result.is_ok(), "Should create refresh token successfully");
        let created = result.unwrap();
        assert_eq!(created.user_id, user_id);
        assert_eq!(created.token_hash, new_token.token_hash);

        // Cleanup
        let _ = RefreshTokenRepository::delete(created.id);
        let _ = UserRepository::delete(user_id);
    }

    // ============================================
    // Test 2: Trouver par hash
    // ============================================
    #[test]
    fn test_find_by_hash_success() {
        // Arrange
        let user_id = create_test_user();  // ← Créer le user
        let new_token = create_test_refresh_token(user_id);
        let hash = new_token.token_hash.clone();

        let created = RefreshTokenRepository::create(&new_token)
            .expect("Failed to create token");

        // Act
        let result = RefreshTokenRepository::find_by_hash(&hash);

        // Assert
        assert!(result.is_ok(), "Should find token by hash");
        let found = result.unwrap();
        assert!(found.is_some(), "Token should exist");
        assert_eq!(found.unwrap().id, created.id);

        // Cleanup
        let _ = RefreshTokenRepository::delete(created.id);
        let _ = UserRepository::delete(user_id);
    }

    // ============================================
    // Test 3: Hash inexistant
    // ============================================
    #[test]
    fn test_find_by_hash_not_found() {
        // Act
        let result = RefreshTokenRepository::find_by_hash("nonexistent_hash_12345");

        // Assert
        assert!(result.is_ok(), "Query should succeed");
        let found = result.unwrap();
        assert!(found.is_none(), "Token should not exist");
    }

    // ============================================
    // Test 4: Token expiré n'est pas trouvé
    // ============================================
    #[test]
    fn test_find_by_hash_expired_token() {
        // Arrange
        let user_id = create_test_user();  // ← Créer le user
        let expired_token = NewRefreshToken {
            user_id,
            token_hash: format!("expired_hash_{}", Uuid::new_v4()),
            expires_at: Utc::now() - chrono::Duration::hours(1),  // ← Expiré
        };

        let created = RefreshTokenRepository::create(&expired_token)
            .expect("Failed to create token");

        // Act
        let result = RefreshTokenRepository::find_by_hash(&expired_token.token_hash);

        // Assert
        assert!(result.is_ok(), "Query should succeed");
        let found = result.unwrap();
        assert!(found.is_none(), "Expired token should not be found");

        // Cleanup
        let _ = RefreshTokenRepository::delete(created.id);
        let _ = UserRepository::delete(user_id);
    }

    // ============================================
    // Test 5: Delete token by ID
    // ============================================
    #[test]
    fn test_delete_by_id_success() {
        // Arrange
        let user_id = create_test_user();  // ← Créer le user
        let new_token = create_test_refresh_token(user_id);
        
        let created = RefreshTokenRepository::create(&new_token)
            .expect("Failed to create token");
        let token_id = created.id;

        // Vérifier qu'il existe
        let before = RefreshTokenRepository::find_by_hash(&new_token.token_hash)
            .expect("Failed to query")
            .expect("Token should exist");
        assert_eq!(before.id, token_id);

        // Act
        let result = RefreshTokenRepository::delete(token_id);

        // Assert
        assert!(result.is_ok(), "Should delete successfully");

        // Vérifier qu'il n'existe plus
        let after = RefreshTokenRepository::find_by_hash(&new_token.token_hash)
            .expect("Failed to query");
        assert!(after.is_none(), "Token should be deleted");

        // Cleanup
        let _ = UserRepository::delete(user_id);
    }
}