use anyhow::anyhow;
use chrono::NaiveDate;
use domain::{
    FutureMeetUp, FutureMeetUpGateway, FutureMeetUpState, GetFutureMeetUpError,
    NewFutureMeetUpError,
};
use sqlx::{sqlite::SqliteRow, Error, Row};
use ulid::Ulid;

use super::SqliteDatabaseGateway;

impl FutureMeetUpGateway for SqliteDatabaseGateway {
    async fn get_future_meet_up(&self) -> Result<Option<FutureMeetUp>, GetFutureMeetUpError> {
        let result = sqlx::query("SELECT * FROM future_meet_ups")
            .try_map(|row: SqliteRow| {
                let state = match row.get::<i32, _>("state") {
                    0 => FutureMeetUpState::CallForPapers,
                    1 => FutureMeetUpState::Voting,
                    2 => FutureMeetUpState::Scheduled {
                        title: row.get("title"),
                        description: row.get("description"),
                        speaker: row.get("speaker"),
                    },
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
            Err(err) => Err(GetFutureMeetUpError::Unknown(anyhow!("SQLX Error: {err}"))),
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
            .map_err(|err| NewFutureMeetUpError::Unknown(anyhow!("SQLX Error: {err}")))?;
        Ok(FutureMeetUp::new(
            id,
            FutureMeetUpState::CallForPapers,
            location,
            date,
        ))
    }
}
