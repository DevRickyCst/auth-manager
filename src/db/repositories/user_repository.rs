use crate::db::connection::get_connection;
use crate::db::error::RepositoryError;
use crate::db::models::user::{NewUser, UpdateUser, User};
use crate::db::schema::users;
use diesel::prelude::*;
use uuid::Uuid;

pub struct UserRepository;

impl UserRepository {
    /// Finds a user by email address. Returns `None` if no match.
    pub fn find_by_email(email: &str) -> Result<Option<User>, RepositoryError> {
        let mut conn = get_connection()?;

        users::table
            .filter(users::email.eq(email))
            .first::<User>(&mut conn)
            .optional()
            .map_err(Into::into)
    }

    /// Trouver un utilisateur par ID
    pub fn find_by_id(id: Uuid) -> Result<Option<User>, RepositoryError> {
        let mut conn = get_connection()?;

        users::table
            .filter(users::id.eq(id))
            .first::<User>(&mut conn)
            .optional()
            .map_err(Into::into)
    }

    /// Créer un nouvel utilisateur
    pub fn create(new_user: &NewUser) -> Result<User, RepositoryError> {
        let mut conn = get_connection()?;

        diesel::insert_into(users::table)
            .values(new_user)
            .get_result::<User>(&mut conn)
            .map_err(Into::into)
    }

    /// Mettre à jour le dernier login
    pub fn update_last_login(id: Uuid) -> Result<(), RepositoryError> {
        let changes = UpdateUser {
            last_login_at: Some(Some(chrono::Utc::now())),
            ..Default::default()
        };
        Self::update(id, &changes)?;
        Ok(())
    }

    /// Replaces the stored password hash for the given user.
    pub fn update_password(id: Uuid, new_password_hash: &str) -> Result<(), RepositoryError> {
        let mut conn = get_connection()?;

        diesel::update(users::table.filter(users::id.eq(id)))
            .set(users::password_hash.eq(new_password_hash))
            .execute(&mut conn)?;

        Ok(())
    }

    /// Mettre à jour un utilisateur (`email_verified`, `is_active`, `last_login_at`)
    pub fn update(id: Uuid, changes: &UpdateUser) -> Result<User, RepositoryError> {
        let mut conn = get_connection()?;

        diesel::update(users::table.filter(users::id.eq(id)))
            .set(changes)
            .get_result::<User>(&mut conn)
            .map_err(Into::into)
    }

    /// Supprimer un utilisateur
    pub fn delete(id: Uuid) -> Result<(), RepositoryError> {
        let mut conn = get_connection()?;

        diesel::delete(users::table.filter(users::id.eq(id))).execute(&mut conn)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::password::PasswordManager;
    use crate::db::connection::init_test_pool;

    fn create_test_user(suffix: &str) -> NewUser {
        init_test_pool();

        NewUser {
            email: format!(
                "test_{}_{:?}@example.com",
                suffix,
                std::time::SystemTime::now()
            ),
            username: format!("testuser_{suffix}"),
            password_hash: Some("test_hash".to_string()),
        }
    }

    mod create {
        use super::*;

        #[test]
        fn create_user_succeeds_with_valid_data() {
            let new_user = create_test_user("create");

            let created_user = UserRepository::create(&new_user).expect("Should create user");

            assert_eq!(created_user.email, new_user.email);
            assert_eq!(created_user.username, new_user.username);

            let _ = UserRepository::delete(created_user.id);
        }

        #[test]
        fn create_fails_when_email_already_exists() {
            init_test_pool();
            let email = format!("duplicate_{}@example.com", Uuid::new_v4());
            let user1 = NewUser {
                email: email.clone(),
                username: "user1".to_string(),
                password_hash: Some("hash".to_string()),
            };
            let user2 = NewUser {
                email,
                username: "user2".to_string(),
                password_hash: Some("hash".to_string()),
            };

            let created1 = UserRepository::create(&user1).expect("Failed to create first user");

            let result = UserRepository::create(&user2);

            assert!(
                result.is_err(),
                "Should fail due to unique constraint on email"
            );

            let _ = UserRepository::delete(created1.id);
        }
    }

    mod find_by_email {
        use super::*;

        #[test]
        fn find_by_email_returns_user_when_exists() {
            let new_user = create_test_user("find_email");
            let created = UserRepository::create(&new_user).expect("Failed to create user");

            let found = UserRepository::find_by_email(&new_user.email)
                .expect("Query should succeed")
                .expect("User should exist");

            assert_eq!(found.id, created.id);

            let _ = UserRepository::delete(created.id);
        }

        #[test]
        fn find_by_email_returns_none_when_not_found() {
            init_test_pool();

            let found = UserRepository::find_by_email("nonexistent_email_12345@example.com")
                .expect("Query should succeed even if user not found");

            assert!(found.is_none(), "User should not exist");
        }
    }

    mod find_by_id {
        use super::*;

        #[test]
        fn find_by_id_returns_user_when_exists() {
            let new_user = create_test_user("find_id");
            let created = UserRepository::create(&new_user).expect("Failed to create user");

            let found = UserRepository::find_by_id(created.id)
                .expect("Query should succeed")
                .expect("User should exist");

            assert_eq!(found.id, created.id);

            let _ = UserRepository::delete(created.id);
        }

        #[test]
        fn find_by_id_returns_none_when_not_found() {
            init_test_pool();

            let found = UserRepository::find_by_id(Uuid::new_v4())
                .expect("Query should succeed even if user not found");

            assert!(found.is_none(), "User should not exist");
        }
    }

    mod update {
        use super::*;

        #[test]
        fn update_last_login_sets_timestamp_in_db() {
            let new_user = create_test_user("login");
            let created = UserRepository::create(&new_user).expect("Failed to create user");
            let user_id = created.id;

            let before = UserRepository::find_by_id(user_id)
                .expect("Query should succeed")
                .expect("User should exist");
            assert!(
                before.last_login_at.is_none(),
                "last_login_at should be None initially"
            );

            UserRepository::update_last_login(user_id).expect("Should update last_login");

            let after = UserRepository::find_by_id(user_id)
                .expect("Query should succeed")
                .expect("User should exist");
            assert!(
                after.last_login_at.is_some(),
                "last_login_at should be set after update"
            );

            let _ = UserRepository::delete(user_id);
        }

        #[test]
        fn update_password_stores_new_hash_in_db() {
            init_test_pool();
            let new_user = NewUser {
                email: format!("update_pw_{}@example.com", Uuid::new_v4()),
                username: "update_pw_user".to_string(),
                password_hash: Some(PasswordManager::hash("OldPass123!").expect("hash")),
            };

            let created = UserRepository::create(&new_user).expect("Failed to create user");
            let user_id = created.id;

            let new_hash = PasswordManager::hash("NewPass456!").expect("hash");
            UserRepository::update_password(user_id, &new_hash).expect("Should update password");

            let updated = UserRepository::find_by_id(user_id)
                .expect("Should find user")
                .expect("User should exist");
            let hash = updated.password_hash.as_ref().expect("hash present");
            let ok = PasswordManager::verify("NewPass456!", hash).expect("verify");
            assert!(ok, "New password should verify");

            let _ = UserRepository::delete(user_id);
        }
    }
}
