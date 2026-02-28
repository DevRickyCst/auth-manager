use super::error::RepositoryError;
use super::{DbConnection, DbPool};
use anyhow::{Context, Result, anyhow};
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use std::sync::OnceLock;

static POOL: OnceLock<DbPool> = OnceLock::new();

/// Initialize the `PostgreSQL` connection pool with the given database URL.
/// This should be called once at application startup.
pub fn init_pool_with_url(database_url: &str) -> Result<()> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = diesel::r2d2::Pool::builder()
        .max_size(15)
        .build(manager)
        .context("Failed to build r2d2 pool")?;

    POOL.set(pool)
        .map_err(|_| anyhow!("Pool already initialized"))?;

    Ok(())
}

/// Initialize the `PostgreSQL` connection pool using `DATABASE_URL` env var.
/// This should be called once at application startup.
#[allow(dead_code)]
pub fn init_pool() -> Result<()> {
    let url = std::env::var("DATABASE_URL").context("DATABASE_URL env var not set")?;
    init_pool_with_url(&url)
}

/// Get a reference to the initialized pool.
///
/// # Panics
///
/// Panics if [`init_pool_with_url`] hasn't been called before this function.
pub fn get_pool() -> &'static DbPool {
    POOL.get()
        .expect("DB pool not initialized. Call init_pool() first.")
}

/// Get a connection from the pool.
/// Returns `RepositoryError` for use in repository layer.
pub fn get_connection() -> Result<DbConnection, RepositoryError> {
    get_pool()
        .get()
        .map_err(|e| RepositoryError::PoolError(e.to_string()))
}

#[cfg(test)]
pub fn init_test_pool() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let url = std::env::var("TEST_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .expect("TEST_DATABASE_URL (or DATABASE_URL) must be set for tests");
        let _ = init_pool_with_url(&url);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_initializes_successfully() {
        init_test_pool();
        // Pool should now be initialized or skip if DATABASE_URL not set
    }
}
