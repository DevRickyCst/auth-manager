pub mod connection;
pub mod error;
pub mod models;
pub mod repositories;
pub mod schema;

use diesel::PgConnection;
use diesel::r2d2::{self, ConnectionManager};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub use connection::{get_connection, get_pool, init_pool};
pub use error::RepositoryError;
