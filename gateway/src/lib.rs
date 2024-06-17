use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use sqlx::{Error, Row, sqlite::SqliteRow, SqlitePool};
use ulid::Ulid;
use url::Url;

use domain::{FutureMeetUp, FutureMeetUpGateway, FutureMeetUpState, GetFutureMeetUpError, GetPastMeetUpError, ListPastMeetUpsError, NewFutureMeetUpError, PastMeetUp, PastMeetUpGateway, PastMeetUpMetadata};

pub struct SqliteDatabaseGateway {
    sqlite_pool: SqlitePool,
}

impl SqliteDatabaseGateway {
    pub async fn new(database_url: String) -> Result<Self> {
        let sqlite_pool = SqlitePool::connect(&database_url).await?;
        sqlx::migrate!("./migrations").run(&sqlite_pool).await?;
        Ok(Self { sqlite_pool })
    }
}

impl PastMeetUpGateway for SqliteDatabaseGateway {
    async fn list_past_meet_ups(&self) -> Result<Vec<PastMeetUpMetadata>, ListPastMeetUpsError> {
        Ok(
            sqlx::query("SELECT id, title, date FROM past_meet_ups ORDER BY date desc;")
                .try_map(|row: SqliteRow| {
                    Ok(PastMeetUpMetadata::new(
                        Ulid::from_bytes(
                            row.try_get::<&[u8], _>("id")?
                                .try_into()
                                .map_err(|err| sqlx::Error::Decode(Box::new(err)))?,
                        ),
                        row.get("title"),
                        row.get("date"),
                    ))
                })
                .fetch_all(&self.sqlite_pool)
                .await
                .map_err(|err| anyhow!("SQLX Error: {err}"))?,
        )
    }

    async fn get_past_meet_up(&self, id: Ulid) -> Result<PastMeetUp, GetPastMeetUpError> {
        sqlx::query("SELECT * FROM past_meet_ups WHERE id = ?")
            .bind(id.to_bytes().as_slice())
            .try_map(|row: SqliteRow| {
                Ok(PastMeetUp::new(
                    Ulid::from_bytes(
                        row.try_get::<&[u8], _>("id")?
                            .try_into()
                            .map_err(|err| Error::Decode(Box::new(err)))?,
                    ),
                    row.get("title"),
                    row.get("description"),
                    row.get("speaker"),
                    row.get("date"),
                    Url::parse(row.get("link")).map_err(|err| Error::Decode(Box::new(err)))?,
                ))
            })
            .fetch_one(&self.sqlite_pool)
            .await
            .map_err(|err| match err {
                Error::RowNotFound => GetPastMeetUpError::NotFound(id),
                _ => GetPastMeetUpError::Unknown(anyhow!("SQLX Error: {err}")),
            })
    }

    async fn get_past_meet_up_metadata(&self, id: Ulid) -> Result<PastMeetUpMetadata, GetPastMeetUpError> {
        sqlx::query("SELECT id, title, date FROM past_meet_ups WHERE id = ?")
            .bind(id.to_bytes().as_slice())
            .try_map(|row: SqliteRow| {
                Ok(PastMeetUpMetadata::new(
                    Ulid::from_bytes(
                        row.try_get::<&[u8], _>("id")?
                            .try_into()
                            .map_err(|err| Error::Decode(Box::new(err)))?,
                    ),
                    row.get("title"),
                    row.get("date"),
                ))
            })
            .fetch_one(&self.sqlite_pool)
            .await
            .map_err(|err| match err {
                Error::RowNotFound => GetPastMeetUpError::NotFound(id),
                _ => GetPastMeetUpError::Unknown(anyhow!("SQLX Error: {err}")),
            })
    }
}

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

    async fn new_future_meet_up(&self, id: Ulid, location: String, date: NaiveDate) -> std::result::Result<FutureMeetUp, NewFutureMeetUpError> {
        sqlx::query(
            "INSERT INTO future_meet_ups (id, state, location, date) VALUES (?, ?, ?, ?)",
        ).bind(id.to_bytes().as_slice())
            .bind(0)
            .bind(&location)
            .bind(&date)
            .execute(&self.sqlite_pool)
            .await
            .map_err(|err| NewFutureMeetUpError::Unknown(anyhow!("SQLX Error: {err}")))?;
        Ok(
            FutureMeetUp::new(
                id,
                FutureMeetUpState::CallForPapers,
                location,
                date,
            )
        )
    }
}
