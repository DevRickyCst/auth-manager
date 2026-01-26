use diesel::{Insertable, Queryable, Selectable};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::db::schema::user_identities;

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = user_identities)]
pub struct NewUserIdentity {
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub email: Option<String>,
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = user_identities)]
pub struct UserIdentity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}