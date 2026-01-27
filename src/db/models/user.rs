use diesel::{Insertable, Queryable, Selectable, AsChangeset};
use chrono::{DateTime, Utc};
use crate::dto::responses::UserResponse;
use uuid::Uuid;
use crate::db::schema::users;

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub username: String,
    pub password_hash: Option<String>,
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: Option<String>,
    pub email_verified: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    #[allow(dead_code)]
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            email_verified: user.email_verified,
            is_active: user.is_active,
            created_at: user.created_at,
        }
    }
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    pub email_verified: Option<bool>,
    pub is_active: Option<bool>,
    pub last_login_at: Option<Option<DateTime<Utc>>>,
}