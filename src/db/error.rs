/// Repository layer errors
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Connection pool error: {0}")]
    PoolError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),
    #[error("Foreign key constraint violation: {0}")]
    ForeignKeyViolation(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<diesel::result::Error> for RepositoryError {
    fn from(err: diesel::result::Error) -> Self {
        use diesel::result::{DatabaseErrorKind, Error};

        match err {
            Error::NotFound => RepositoryError::NotFound("Record not found".to_string()),
            Error::DatabaseError(kind, info) => {
                let message = info.message().to_string();
                match kind {
                    DatabaseErrorKind::UniqueViolation => RepositoryError::UniqueViolation(message),
                    DatabaseErrorKind::ForeignKeyViolation => {
                        RepositoryError::ForeignKeyViolation(message)
                    }
                    _ => RepositoryError::DatabaseError(message),
                }
            }
            _ => RepositoryError::DatabaseError(err.to_string()),
        }
    }
}

impl From<diesel::r2d2::PoolError> for RepositoryError {
    fn from(err: diesel::r2d2::PoolError) -> Self {
        RepositoryError::PoolError(err.to_string())
    }
}

// Note: This implementation is redundant with std::error::Error
// Anyhow already provides From<E> for any E: std::error::Error
// We rely on that blanket implementation instead of defining our own
