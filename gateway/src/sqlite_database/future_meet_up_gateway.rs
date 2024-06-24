use crate::error_and_log;
use chrono::{NaiveDate, Utc};
use domain::{
    FutureMeetUp, FutureMeetUpGateway, FutureMeetUpState, GetFutureMeetUpError,
    NewFutureMeetUpError, Paper, UpdateFutureMeetUpError,
};
use sqlx::{sqlite::SqliteRow, Error, Row};
use ulid::Ulid;
use url::Url;

use super::SqliteDatabaseGateway;

impl FutureMeetUpGateway for SqliteDatabaseGateway {
    async fn get_future_meet_up(&self) -> Result<Option<FutureMeetUp>, GetFutureMeetUpError> {
        let result = sqlx::query(
            "SELECT fmu.*, p.id as paper_id, p.title, p.description, p.speaker, p.user_id, p.email FROM future_meet_ups fmu LEFT JOIN papers p ON p.id = fmu.paper_id;",
        )
        .try_map(|row: SqliteRow| {
            let state = match row.get::<i32, _>("state") {
                0 => FutureMeetUpState::CallForPapers,
                1 => FutureMeetUpState::Voting,
                2 => FutureMeetUpState::Scheduled (
                    Paper {
                        id: Ulid::from_bytes(
                            row.try_get::<&[u8], _>("paper_id")?
                                .try_into()
                                .map_err(|err| Error::Decode(Box::new(err)))?,
                        ),
                        user_id: Ulid::from_bytes(
                            row.try_get::<&[u8], _>("user_id")?
                                .try_into()
                                .map_err(|err| Error::Decode(Box::new(err)))?,
                        ),
                        title: row.get("title"),
                        description: row.get("description"),
                        speaker: row.get("speaker"),
                        email: row.get("email"),
                    }
                ),
                _ => return Err(Error::Decode("Unknown state".into())),
            };
            Ok(FutureMeetUp::new(
                Ulid::from_bytes(
                    row.try_get::<&[u8], _>("id")?
                        .try_into()
                        .map_err(|err| Error::Decode(Box::new(err)))?,
                ),
                state,
                row.get("location"),
                row.get("date"),
            ))
        })
        .fetch_one(&self.sqlite_pool)
        .await;
        match result {
            Ok(future_meet_up) => Ok(Some(future_meet_up)),
            Err(Error::RowNotFound) => Ok(None),
            Err(err) => Err(GetFutureMeetUpError::Unknown(error_and_log!(
                "SQLX Error: {err}"
            ))),
        }
    }

    async fn new_future_meet_up(
        &self,
        id: Ulid,
        location: String,
        date: NaiveDate,
    ) -> Result<FutureMeetUp, NewFutureMeetUpError> {
        sqlx::query("INSERT INTO future_meet_ups (id, state, location, date) VALUES (?, ?, ?, ?)")
            .bind(id.to_bytes().as_slice())
            .bind(0)
            .bind(&location)
            .bind(date)
            .execute(&self.sqlite_pool)
            .await
            .map_err(|err| NewFutureMeetUpError::Unknown(error_and_log!("SQLX Error: {err}")))?;
        Ok(FutureMeetUp::new(
            id,
            FutureMeetUpState::CallForPapers,
            location,
            date,
        ))
    }

    async fn update_future_meet_up_to_voting(
        &self,
        id: &Ulid,
    ) -> Result<FutureMeetUp, UpdateFutureMeetUpError> {
        let rows_affected = sqlx::query(
            "UPDATE future_meet_ups SET state = 1, updated_at = ? WHERE id = ? AND state = 0",
        )
        .bind(Utc::now())
        .bind(id.to_bytes().as_slice())
        .execute(&self.sqlite_pool)
        .await
        .map_err(|err| UpdateFutureMeetUpError::Unknown(error_and_log!("SQLX Error: {err}")))?
        .rows_affected();
        let future_meet_up = self
            .get_future_meet_up()
            .await
            .map_err(|err| error_and_log!("{err}"))?
            .ok_or(UpdateFutureMeetUpError::NotFound)?;
        if rows_affected == 0 {
            return Err(UpdateFutureMeetUpError::InvalidState);
        }

        Ok(future_meet_up)
    }

    async fn update_future_meet_up_to_scheduled(
        &self,
        id: &Ulid,
        paper_id: &Ulid,
    ) -> Result<FutureMeetUp, UpdateFutureMeetUpError> {
        let rows_affected = sqlx::query(
            "UPDATE future_meet_ups SET state = 2, paper_id = ?, updated_at = ? WHERE id = ? AND state = 1",
        )
        .bind(paper_id.to_bytes().as_slice())
        .bind(Utc::now())
        .bind(id.to_bytes().as_slice())
        .execute(&self.sqlite_pool)
        .await
        .map_err(|err| UpdateFutureMeetUpError::Unknown(error_and_log!("SQLX Error: {err}")))?
        .rows_affected();
        let future_meet_up = self
            .get_future_meet_up()
            .await
            .map_err(|err| error_and_log!("{err}"))?
            .ok_or(UpdateFutureMeetUpError::NotFound)?;
        if rows_affected == 0 {
            return Err(UpdateFutureMeetUpError::InvalidState);
        }

        Ok(future_meet_up)
    }

    async fn finish_future_meet_up(
        &self,
        id: &Ulid,
        link: Url,
    ) -> Result<(), UpdateFutureMeetUpError> {
        let mut transaction =
            self.sqlite_pool.begin().await.map_err(|err| {
                UpdateFutureMeetUpError::Unknown(error_and_log!("SQLX Error: {err}"))
            })?;
        let affected_rows = sqlx::query(r#"
            INSERT INTO past_meet_ups (id, paper_id, date, link, location, created_at, updated_at)
            SELECT id, paper_id, date, ? as link, location, created_at, ? as updated_at FROM future_meet_ups WHERE state = 2 AND id = ?;"#
        ).bind(link.as_str())
        .bind(Utc::now())
        .bind(id.to_bytes().as_slice()).execute(&mut *transaction)
        .await
        .map_err(|err| UpdateFutureMeetUpError::Unknown(error_and_log!("SQLX Error: {err}")))?
        .rows_affected();
        if affected_rows != 1 {
            return Err(UpdateFutureMeetUpError::InvalidState);
        }
        let affected_rows = sqlx::query("DELETE FROM future_meet_ups WHERE id = ?")
            .bind(id.to_bytes().as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|err| UpdateFutureMeetUpError::Unknown(error_and_log!("SQLX Error: {err}")))?
            .rows_affected();
        if affected_rows != 1 {
            return Err(UpdateFutureMeetUpError::NotFound);
        }
        transaction
            .commit()
            .await
            .map_err(|err| UpdateFutureMeetUpError::Unknown(error_and_log!("SQLX Error: {err}")))?;
        Ok(())
    }
}
