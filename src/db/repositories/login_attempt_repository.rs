use crate::db::connection::get_connection;
use crate::db::error::RepositoryError;
use crate::db::models::login_attempt::{LoginAttempt, NewLoginAttempt};
use crate::db::schema::login_attempts;
use diesel::prelude::*;
use uuid::Uuid;
pub struct LoginAttemptRepository;

impl LoginAttemptRepository {
    /// Créer une tentative de login
    pub fn create(
        user_id: Option<Uuid>,
        success: bool,
        user_agent: Option<String>,
    ) -> Result<LoginAttempt, RepositoryError> {
        let mut conn = get_connection()?;

        let new_attempt = NewLoginAttempt {
            user_id: &user_id,
            success,
            user_agent: &user_agent,
        };

        diesel::insert_into(login_attempts::table)
            .values(new_attempt)
            .get_result::<LoginAttempt>(&mut conn)
            .map_err(Into::into)
    }

    /// Compter les tentatives échouées pour un user dans les X dernières minutes
    #[allow(dead_code)]
    pub fn count_failed_attempts(user_id: Uuid, minutes: i64) -> Result<i64, RepositoryError> {
        let mut conn = get_connection()?;

        login_attempts::table
            .filter(login_attempts::user_id.eq(user_id))
            .filter(login_attempts::success.eq(false))
            .filter(
                login_attempts::attempted_at
                    .gt(chrono::Utc::now() - chrono::Duration::minutes(minutes)),
            )
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(Into::into)
    }

    /// Récupérer les dernières tentatives d'un user
    #[allow(dead_code)]
    pub fn find_by_user(user_id: Uuid, limit: i64) -> Result<Vec<LoginAttempt>, RepositoryError> {
        let mut conn = get_connection()?;

        login_attempts::table
            .filter(login_attempts::user_id.eq(user_id))
            .order_by(login_attempts::attempted_at.desc())
            .limit(limit)
            .load::<LoginAttempt>(&mut conn)
            .map_err(Into::into)
    }
}
