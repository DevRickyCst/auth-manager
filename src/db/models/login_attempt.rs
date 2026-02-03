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

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = login_attempts)]
pub struct LoginAttempt {
    #[allow(dead_code)]
    pub id: Uuid,
    #[allow(dead_code)]
    pub user_id: Option<Uuid>,
    #[allow(dead_code)]
    pub success: bool,
    #[allow(dead_code)]
    pub attempted_at: DateTime<Utc>,
    #[allow(dead_code)]
    pub user_agent: Option<String>,
}
