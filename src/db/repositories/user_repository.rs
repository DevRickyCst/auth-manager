use crate::db::connection::get_connection;
use crate::db::models::user::{User, NewUser};
use diesel::prelude::*;
use uuid::Uuid;
use crate::db::schema::users;
use crate::db::error::{RepositoryError, map_diesel_error};

pub struct UserRepository;

impl UserRepository {

    pub fn find_by_email(email: &str) -> Result<Option<User>, RepositoryError> {
        let mut conn = get_connection()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        
        users::table
            .filter(users::email.eq(email))
            .first::<User>(&mut *conn)
            .optional()
            .map_err(map_diesel_error)
    }

    /// Trouver un utilisateur par ID
    pub fn find_by_id(id: Uuid) -> Result<Option<User>, RepositoryError> {
        let mut conn = get_connection()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        
        users::table
            .filter(users::id.eq(id))
            .first::<User>(&mut conn)
            .optional()
            .map_err(map_diesel_error)
    }

    /// Créer un nouvel utilisateur
    pub fn create(new_user: &NewUser) -> Result<User, RepositoryError> {
        let mut conn = get_connection()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        
        diesel::insert_into(users::table)
            .values(new_user)
            .get_result::<User>(&mut conn)
            .map_err(map_diesel_error)
    }

    /// Mettre à jour le dernier login
    pub fn update_last_login(id: Uuid) -> Result<(), RepositoryError> {
        let mut conn = get_connection()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        
        diesel::update(users::table.filter(users::id.eq(id)))
            .set(users::last_login_at.eq(chrono::Utc::now()))
            .execute(&mut conn)
            .map_err(map_diesel_error)?;
        
        Ok(())
    }

    /// Supprimer un utilisateur
    pub fn delete(id: Uuid) -> Result<(), RepositoryError> {
        let mut conn = get_connection()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        
        diesel::delete(users::table.filter(users::id.eq(id)))
            .execute(&mut conn)
            .map_err(map_diesel_error)?;
        
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user(suffix: &str) -> NewUser {
        NewUser {
            email: format!("test_{}_{:?}@example.com", suffix, std::time::SystemTime::now()),
            username: format!("testuser_{}", suffix),
            password_hash: Some("test_hash".to_string()),
        }
    }

    // ============================================
    // Test 1: Créer un utilisateur
    // ============================================
    #[test]
    fn test_create_user_success() {
        let new_user = create_test_user("create");

        let result = UserRepository::create(&new_user);

        assert!(result.is_ok(), "Should create user successfully");
        let created_user = result.unwrap();
        assert_eq!(created_user.email, new_user.email);
        assert_eq!(created_user.username, new_user.username);

        // Cleanup
        let _ = UserRepository::delete(created_user.id);
    }

    // ============================================
    // Test 2: Trouver par email
    // ============================================
    #[test]
    fn test_find_by_email_success() {
        let new_user = create_test_user("find_email");
        let created = UserRepository::create(&new_user).expect("Failed to create user");

        let result = UserRepository::find_by_email(&new_user.email);

        assert!(result.is_ok(), "Should find user by email");
        let found = result.unwrap();
        assert!(found.is_some(), "User should exist");
        assert_eq!(found.unwrap().id, created.id);

        // Cleanup
        let _ = UserRepository::delete(created.id);
    }

    // ============================================
    // Test 3: Email non existant
    // ============================================
    #[test]
    fn test_find_by_email_not_found() {
        let result = UserRepository::find_by_email("nonexistent_email_12345@example.com");

        assert!(result.is_ok(), "Query should succeed even if user not found");
        let found = result.unwrap();
        assert!(found.is_none(), "User should not exist");
    }

    // ============================================
    // Test 4: Trouver par ID
    // ============================================
    #[test]
    fn test_find_by_id_success() {
        let new_user = create_test_user("find_id");
        let created = UserRepository::create(&new_user).expect("Failed to create user");

        let result = UserRepository::find_by_id(created.id);

        assert!(result.is_ok(), "Should find user by id");
        let found = result.unwrap();
        assert!(found.is_some(), "User should exist");
        assert_eq!(found.unwrap().id, created.id);

        // Cleanup
        let _ = UserRepository::delete(created.id);
    }

    // ============================================
    // Test 5: ID non existant
    // ============================================
    #[test]
    fn test_find_by_id_not_found() {
        let random_id = Uuid::new_v4();

        let result = UserRepository::find_by_id(random_id);

        assert!(result.is_ok(), "Query should succeed even if user not found");
        let found = result.unwrap();
        assert!(found.is_none(), "User should not exist");
    }

    // ============================================
    // Test 6: Update last login
    // ============================================
    #[test]
    fn test_update_last_login_success() {
        let new_user = create_test_user("login");
        let created = UserRepository::create(&new_user).expect("Failed to create user");
        let user_id = created.id;

        let before = UserRepository::find_by_id(user_id)
            .expect("Should find user")
            .expect("User should exist");
        assert!(before.last_login_at.is_none(), "last_login_at should be None initially");

        let result = UserRepository::update_last_login(user_id);

        assert!(result.is_ok(), "Should update last_login successfully");

        let after = UserRepository::find_by_id(user_id)
            .expect("Should find user")
            .expect("User should exist");
        assert!(after.last_login_at.is_some(), "last_login_at should be set");

        // Cleanup
        let _ = UserRepository::delete(user_id);
    }

    // ============================================
    // Test 7: Duplicate email
    // ============================================
    #[test]
    fn test_create_duplicate_email_fails() {
        let email = format!("duplicate_{}@example.com", Uuid::new_v4());
        let user1 = NewUser {
            email: email.clone(),
            username: "user1".to_string(),
            password_hash: Some("hash".to_string()),
        };
        let user2 = NewUser {
            email: email.clone(),
            username: "user2".to_string(),
            password_hash: Some("hash".to_string()),
        };

        let created1 = UserRepository::create(&user1).expect("Failed to create first user");

        let result = UserRepository::create(&user2);

        assert!(result.is_err(), "Should fail due to unique constraint on email");

        // Cleanup
        let _ = UserRepository::delete(created1.id);
    }
}