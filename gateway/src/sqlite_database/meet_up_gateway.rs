use chrono::NaiveDate;
use sqlx::{sqlite::SqliteRow, Error, Row};
use ulid::Ulid;
use url::Url;

use domain::{
    GetFutureMeetUpError, GetMeetUpError, ListPastMeetUpsError, MeetUp, MeetUpGateway,
    MeetUpMetadata, MeetUpState, NewMeetUpError, Paper, UpdateMeetUpError,
};
use shared::utc_now;

use crate::error_and_log;

use super::SqliteDatabaseGateway;

impl MeetUpGateway for SqliteDatabaseGateway {
    async fn get_future_meet_up(&self) -> Result<Option<MeetUp>, GetFutureMeetUpError> {
        let result = sqlx::query(
            "SELECT mu.*, p.id as paper_id, p.title, p.description, p.speaker, p.user_id, p.email FROM meet_ups mu LEFT JOIN papers p ON p.id = mu.paper_id WHERE mu.state != 3;",
        )
            .try_map(meet_up_from_sqlite_row)
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

    async fn new_meet_up(
        &self,
        id: Ulid,
        location: String,
        date: NaiveDate,
    ) -> Result<MeetUp, NewMeetUpError> {
        sqlx::query("INSERT INTO meet_ups (id, state, location, date) VALUES (?, ?, ?, ?)")
            .bind(id.to_bytes().as_slice())
            .bind(0)
            .bind(&location)
            .bind(date)
            .execute(&self.sqlite_pool)
            .await
            .map_err(|err| NewMeetUpError::Unknown(error_and_log!("SQLX Error: {err}")))?;
        Ok(MeetUp::new(id, MeetUpState::CallForPapers, location, date))
    }

    async fn update_meet_up_to_voting(&self, id: &Ulid) -> Result<MeetUp, UpdateMeetUpError> {
        let rows_affected =
            sqlx::query("UPDATE meet_ups SET state = 1, updated_at = ? WHERE id = ? AND state = 0")
                .bind(utc_now())
                .bind(id.to_bytes().as_slice())
                .execute(&self.sqlite_pool)
                .await
                .map_err(|err| UpdateMeetUpError::Unknown(error_and_log!("SQLX Error: {err}")))?
                .rows_affected();
        let meet_up = self
            .get_future_meet_up()
            .await
            .map_err(|err| error_and_log!("{err}"))?
            .ok_or(UpdateMeetUpError::NotFound(*id))?;
        if rows_affected == 0 {
            return Err(UpdateMeetUpError::InvalidState);
        }

        Ok(meet_up)
    }

    async fn update_meet_up_to_scheduled(
        &self,
        id: &Ulid,
        paper_id: &Ulid,
    ) -> Result<MeetUp, UpdateMeetUpError> {
        let rows_affected = sqlx::query(
            "UPDATE meet_ups SET state = 2, paper_id = ?, updated_at = ? WHERE id = ? AND state = 1",
        )
            .bind(paper_id.to_bytes().as_slice())
            .bind(utc_now())
            .bind(id.to_bytes().as_slice())
            .execute(&self.sqlite_pool)
            .await
            .map_err(|err| UpdateMeetUpError::Unknown(error_and_log!("SQLX Error: {err}")))?
            .rows_affected();
        let meet_up = self
            .get_future_meet_up()
            .await
            .map_err(|err| error_and_log!("{err}"))?
            .ok_or(UpdateMeetUpError::NotFound(*id))?;
        if rows_affected == 0 {
            return Err(UpdateMeetUpError::InvalidState);
        }

        Ok(meet_up)
    }

    async fn finish_meet_up(&self, id: &Ulid, link: Url) -> Result<(), UpdateMeetUpError> {
        let rows_affected = sqlx::query(
            "UPDATE meet_ups SET state = 3, link = ?, updated_at = ? WHERE id = ? AND state = 2",
        )
        .bind(link.as_str())
        .bind(utc_now())
        .bind(id.to_bytes().as_slice())
        .execute(&self.sqlite_pool)
        .await
        .map_err(|err| UpdateMeetUpError::Unknown(error_and_log!("SQLX Error: {err}")))?
        .rows_affected();
        if rows_affected == 0 {
            return Err(UpdateMeetUpError::InvalidState);
        }
        Ok(())
    }

    async fn list_past_meet_ups(&self) -> Result<Vec<MeetUpMetadata>, ListPastMeetUpsError> {
        Ok(
            sqlx::query("SELECT mu.id, p.title, date FROM meet_ups mu JOIN papers p ON mu.paper_id = p.id AND mu.state = 3 ORDER BY date desc;")
                .try_map(|row: SqliteRow| {
                    Ok(MeetUpMetadata::new(
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

    async fn get_meet_up(&self, id: &Ulid) -> Result<MeetUp, GetMeetUpError> {
        sqlx::query(
            "SELECT mu.id, mu.paper_id, mu.state, p.user_id, p.title, p.description, p.speaker, p.email, mu.date, mu.link, mu.location FROM meet_ups mu LEFT JOIN papers p ON mu.paper_id = p.id WHERE mu.id = ?",
        )
            .bind(id.to_bytes().as_slice())
            .try_map(meet_up_from_sqlite_row)
            .fetch_one(&self.sqlite_pool)
            .await
            .map_err(|err| match err {
                Error::RowNotFound => GetMeetUpError::NotFound(*id),
                _ => GetMeetUpError::Unknown(error_and_log!("SQLX Error: {err}")),
            })
    }

    async fn get_meet_up_metadata(&self, id: Ulid) -> Result<MeetUpMetadata, GetMeetUpError> {
        sqlx::query("SELECT mu.id, p.title, date FROM meet_ups mu JOIN papers p ON mu.paper_id = p.id AND state = 3 WHERE mu.id = ?")
            .bind(id.to_bytes().as_slice())
            .try_map(|row: SqliteRow| {
                Ok(MeetUpMetadata::new(
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
                Error::RowNotFound => GetMeetUpError::NotFound(id),
                _ => GetMeetUpError::Unknown(error_and_log!("SQLX Error: {err}")),
            })
    }
}
fn meet_up_from_sqlite_row(row: SqliteRow) -> Result<MeetUp, Error> {
    let state = state_from_row(&row)?;
    Ok(MeetUp::new(
        Ulid::from_bytes(
            row.try_get::<&[u8], _>("id")?
                .try_into()
                .map_err(|err| Error::Decode(Box::new(err)))?,
        ),
        state,
        row.get("location"),
        row.get("date"),
    ))
}
fn state_from_row(row: &SqliteRow) -> Result<MeetUpState, Error> {
    Ok(match row.get::<i32, _>("state") {
        0 => MeetUpState::CallForPapers,
        1 => MeetUpState::Voting,
        2 => MeetUpState::Scheduled(Paper {
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
        }),
        3 => MeetUpState::Done {
            paper: Paper {
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
            },
            link: Url::parse(row.get("link")).map_err(|err| Error::Decode(Box::new(err)))?,
        },
        _ => return Err(Error::Decode("Unknown state".into())),
    })
}
