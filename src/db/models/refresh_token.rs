use diesel::{Insertable, Queryable, Selectable};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::db::schema::refresh_tokens;

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = refresh_tokens)]
pub struct NewRefreshToken {
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = refresh_tokens)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
