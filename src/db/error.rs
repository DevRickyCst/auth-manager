use std::fmt;

/// Repository layer errors
#[derive(Debug)]
pub enum RepositoryError {
    PoolError(String),
    NotFound(String),
    UniqueViolation(String),
    ForeignKeyViolation(String),
    DatabaseError(String),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepositoryError::PoolError(msg) => write!(f, "Connection pool error: {}", msg),
            RepositoryError::NotFound(msg) => write!(f, "Not found: {}", msg),
            RepositoryError::UniqueViolation(msg) => {
                write!(f, "Unique constraint violation: {}", msg)
            }
            RepositoryError::ForeignKeyViolation(msg) => {
                write!(f, "Foreign key constraint violation: {}", msg)
            }
            RepositoryError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for RepositoryError {}

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
