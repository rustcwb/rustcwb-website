use crate::error_and_log;
use domain::{
    GetPastMeetUpError, ListPastMeetUpsError, PastMeetUp, PastMeetUpGateway, PastMeetUpMetadata,
};
use sqlx::{sqlite::SqliteRow, Error, Row};
use ulid::Ulid;
use url::Url;

use super::SqliteDatabaseGateway;

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
                .map_err(|err| error_and_log!("SQLX Error: {err}"))?,
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
                _ => GetPastMeetUpError::Unknown(error_and_log!("SQLX Error: {err}")),
            })
    }

    async fn get_past_meet_up_metadata(
        &self,
        id: Ulid,
    ) -> Result<PastMeetUpMetadata, GetPastMeetUpError> {
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
                _ => GetPastMeetUpError::Unknown(error_and_log!("SQLX Error: {err}")),
            })
    }
}
