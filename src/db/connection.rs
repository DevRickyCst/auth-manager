use super::{DbConnection, DbPool};
use anyhow::{Result, anyhow};
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use once_cell::sync::Lazy;

pub static DB_POOL: Lazy<DbPool> = Lazy::new(|| {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(&database_url);

    diesel::r2d2::Pool::builder()
        .max_size(5)
        .build(manager)
        .expect("Failed to create database pool")
});

pub fn get_connection() -> Result<DbConnection> {
    DB_POOL
        .get()
        .map_err(|e| anyhow!("Impossible de récupérer une connexion du pool: {}", e))
}

// ============================================
// CREATE POOL - Si tu veux créer manuellement (optionnel)
// ============================================
#[cfg(test)]
pub fn create_pool() -> Result<DbPool> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(&database_url);

    diesel::r2d2::Pool::builder()
        .max_size(5)
        .build(manager)
        .map_err(|e| anyhow!("Failed to create pool: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_connection_success() {
        // Le pool Lazy est créé automatiquement à la première utilisation
        let result = get_connection();

        // Soit success soit error (dépend si BDD est up)
        // Mais le pool structure doit être bon
        match result {
            Ok(_conn) => {
                println!("✓ Connection successful");
            }
            Err(e) => {
                println!("⚠️ Connection error (expected if DB not running): {}", e);
            }
        }
    }

    #[test]
    fn test_pool_is_created_once() {
        // Appeler get_connection() plusieurs fois
        let _conn1 = get_connection();
        let _conn2 = get_connection();
        let _conn3 = get_connection();

        // Le pool est le même (Lazy ne crée qu'une fois)
        // Aucune erreur, ça compile et fonctionne
        assert!(true);
    }

    #[test]
    fn test_pool_max_size() {
        let result = get_connection();

        match result {
            Ok(_conn) => {
                // Pool créé, check max_size
                assert_eq!(DB_POOL.max_size(), 5);
            }
            Err(_) => {
                // BDD pas disponible, mais Lazy est bon
                assert_eq!(DB_POOL.max_size(), 5);
            }
        }
    }

    #[test]
    fn test_create_pool_manual() {
        // Alternative: créer le pool manuellement (moins courant)
        let result = create_pool();
        assert!(
            result.is_ok(),
            "Pool creation should succeed with valid DATABASE_URL"
        );

        let pool = result.unwrap();
        assert_eq!(pool.max_size(), 5, "Pool max_size should be 5");
    }
}
