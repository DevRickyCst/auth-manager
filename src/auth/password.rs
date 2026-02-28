use bcrypt::{DEFAULT_COST, hash, verify};

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("Password hashing failed: {0}")]
    HashingFailed(bcrypt::BcryptError),
    #[error("Password verification failed: {0}")]
    VerificationFailed(bcrypt::BcryptError),
}

pub struct PasswordManager;

impl PasswordManager {
    pub fn hash(password: &str) -> Result<String, PasswordError> {
        hash(password, DEFAULT_COST).map_err(PasswordError::HashingFailed)
    }

    pub fn verify(password: &str, hash: &str) -> Result<bool, PasswordError> {
        verify(password, hash).map_err(PasswordError::VerificationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::PasswordManager;

    #[test]
    fn verify_returns_true_when_password_matches() {
        let password = "secure_password_@123P";
        let hashed = PasswordManager::hash(password).expect("Hashing failed");

        assert!(PasswordManager::verify(password, &hashed).expect("Verification failed"));
    }

    #[test]
    fn verify_returns_false_when_password_does_not_match() {
        let password = "secure_password_@123P";
        let hashed = PasswordManager::hash(password).expect("Hashing failed");

        assert!(
            !PasswordManager::verify("wrong_password_@123", &hashed).expect("Verification failed")
        );
    }

    #[test]
    fn hashes_differ_for_different_passwords() {
        let hash1 = PasswordManager::hash("user1_password").unwrap();
        let hash2 = PasswordManager::hash("user2_password").unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn cross_verify_rejects_mismatched_password_and_hash() {
        let password1 = "user1_password";
        let password2 = "user2_password";

        let hash1 = PasswordManager::hash(password1).unwrap();
        let hash2 = PasswordManager::hash(password2).unwrap();

        assert!(!PasswordManager::verify(password1, &hash2).unwrap());
        assert!(!PasswordManager::verify(password2, &hash1).unwrap());
    }

    #[test]
    fn verify_fails_when_case_differs() {
        let password = "MyPassword";
        let hash = PasswordManager::hash(password).unwrap();

        let wrong_case = "mypassword";
        let result = PasswordManager::verify(wrong_case, &hash);

        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should be false, not error
    }
}
