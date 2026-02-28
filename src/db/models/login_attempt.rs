use crate::db::schema::login_attempts;
use chrono::{DateTime, Utc};
use diesel::{Insertable, Queryable, Selectable};
use uuid::Uuid;

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = login_attempts)]
pub struct NewLoginAttempt<'a> {
    pub user_id: &'a Option<Uuid>,
    pub success: bool,
    pub user_agent: &'a Option<String>,
}

// All fields are required for Diesel Queryable deserialization (schema alignment).
#[allow(dead_code)]
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = login_attempts)]
pub struct LoginAttempt {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub success: bool,
    pub attempted_at: DateTime<Utc>,
    pub user_agent: Option<String>,
}
