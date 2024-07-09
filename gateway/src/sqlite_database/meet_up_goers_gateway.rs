use sqlx::Row;
use ulid::Ulid;

use domain::{GetAttendeesError, MeetUpGoersGateway, RegisterUserError};

use crate::{error_and_log, SqliteDatabaseGateway};

impl MeetUpGoersGateway for SqliteDatabaseGateway {
    async fn register_user_to_meet_up(
        &self,
        user_id: &Ulid,
        meet_up_id: &Ulid,
    ) -> Result<(), RegisterUserError> {
        sqlx::query("INSERT INTO meet_up_goers (user_id, meet_up_id) VALUES (?, ?)")
            .bind(user_id.to_bytes().as_slice())
            .bind(meet_up_id.to_bytes().as_slice())
            .execute(&self.sqlite_pool)
            .await
            .map_err(|err| error_and_log!("SQLX Error: {err}"))?;
        Ok(())
    }

    async fn is_user_registered_to_meet_up(
        &self,
        user_id: &Ulid,
        meet_up_id: &Ulid,
    ) -> Result<bool, RegisterUserError> {
        Ok(
            sqlx::query("SELECT COUNT(1) FROM meet_up_goers WHERE user_id = ? AND meet_up_id = ?")
                .bind(user_id.to_bytes().as_slice())
                .bind(meet_up_id.to_bytes().as_slice())
                .fetch_one(&self.sqlite_pool)
                .await
                .map(|row| row.get::<i64, _>(0) > 0)
                .map_err(|err| error_and_log!("SQLX Error: {err}"))?,
        )
    }

    async fn get_number_attendees_from_meet_up(
        &self,
        meet_up_id: &Ulid,
    ) -> Result<usize, GetAttendeesError> {
        Ok(
            sqlx::query("SELECT COUNT(1) FROM meet_up_goers WHERE meet_up_id = ?")
                .bind(meet_up_id.to_bytes().as_slice())
                .fetch_one(&self.sqlite_pool)
                .await
                .map(|row| row.get::<i64, _>(0) as usize)
                .map_err(|err| error_and_log!("SQLX Error: {err}"))?,
        )
    }
}
