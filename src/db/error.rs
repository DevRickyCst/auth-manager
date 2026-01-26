use diesel::result::{DatabaseErrorKind, Error as DieselError};

#[derive(Debug)]
pub enum RepositoryError {
    NotFound,
    Duplicate,
    Database(String),
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::NotFound => write!(f, "Not found"),
            RepositoryError::Duplicate => write!(f, "Duplicate entry"),
            RepositoryError::Database(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for RepositoryError {}

pub fn map_diesel_error(e: DieselError) -> RepositoryError {
    match e {
        DieselError::NotFound => RepositoryError::NotFound,
        DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => RepositoryError::Duplicate,
        other => RepositoryError::Database(other.to_string()),
    }
}