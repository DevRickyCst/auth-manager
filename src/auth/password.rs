use bcrypt::{hash, verify, DEFAULT_COST};

pub struct PasswordManager;

impl PasswordManager {
    pub fn hash(password: &str) -> Result<String, String> {
        hash(password, DEFAULT_COST)
            .map_err(|e| format!("Password hashing failed: {}", e))
    }

    pub fn verify(password: &str, hash: &str) -> Result<bool, String> {
        verify(password, hash)
            .map_err(|e| format!("Password verification failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::PasswordManager;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "secure_password_@123P";
        let hashed = PasswordManager::hash(password).expect("Hashing failed");
        
        assert!(PasswordManager::verify(password, &hashed).expect("Verification failed"));
        assert!(!PasswordManager::verify("wrong_password_@123", &hashed).expect("Verification failed"));
    }

    #[test]
    fn test_multiple_users_different_hashes() {
        let password1 = "user1_password";
        let password2 = "user2_password";
        
        let hash1 = PasswordManager::hash(password1).unwrap();
        let hash2 = PasswordManager::hash(password2).unwrap();
        
        // Les hashes doivent être différents
        assert_ne!(hash1, hash2);
        
        // Vérification croisée ne doit pas fonctionner
        assert!(!PasswordManager::verify(password1, &hash2).unwrap());
        assert!(!PasswordManager::verify(password2, &hash1).unwrap());
    }
    #[test]
    fn test_verify_case_sensitive() {
        let password = "MyPassword";
        let hash = PasswordManager::hash(password).unwrap();
        
        let wrong_case = "mypassword";
        let result = PasswordManager::verify(wrong_case, &hash);
        
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should be false, not error
    }
}