use crate::db::schema::refresh_tokens;
use chrono::{DateTime, Utc};
use diesel::{Insertable, Queryable, Selectable};
use uuid::Uuid;

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
    #[expect(dead_code, reason = "Required for Diesel Queryable deserialization")]
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    #[expect(dead_code, reason = "Required for Diesel Queryable deserialization")]
    pub created_at: DateTime<Utc>,
    #[expect(dead_code, reason = "Required for Diesel Queryable deserialization")]
    pub updated_at: DateTime<Utc>,
}
