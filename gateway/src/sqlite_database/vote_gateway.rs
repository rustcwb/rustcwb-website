use chrono::Utc;
use sqlx::{Error, Row, sqlite::SqliteRow};
use ulid::Ulid;

use domain::{Vote, VoteError, VoteGateway};

use crate::{error_and_log, SqliteDatabaseGateway};

impl VoteGateway for SqliteDatabaseGateway {
    async fn store_votes(&self, votes: Vec<Vote>) -> Result<(), VoteError> {
        let mut transaction = self
            .sqlite_pool
            .begin()
            .await
            .map_err(|err| error_and_log!("SQLX Error: `{err}`"))?;
        for vote in votes {
            let now = Utc::now();
            sqlx::query(
                r#"
                INSERT INTO meet_up_papers_votes (user_id, paper_id, meet_up_id, vote, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?)
                ON CONFLICT(user_id, paper_id, meet_up_id) DO UPDATE SET vote=excluded.vote, updated_at=excluded.updated_at;
                "#,
            )
                .bind(vote.user_id.to_bytes().as_slice())
                .bind(vote.paper_id.to_bytes().as_slice())
                .bind(vote.meet_up_id.to_bytes().as_slice())
                .bind(vote.vote)
                .bind(now)
                .bind(now)
                .execute(&mut *transaction)
                .await
                .map_err(|err| error_and_log!("SQLX Error: `{err}`"))?;
        }
        transaction
            .commit()
            .await
            .map_err(|err| error_and_log!("SQLX Error: `{err}`"))?;
        Ok(())
    }

    async fn get_votes_for_user(
        &self,
        meet_up_id: &Ulid,
        user_id: &Ulid,
    ) -> Result<Vec<Vote>, VoteError> {
        let votes =
            sqlx::query("SELECT * FROM meet_up_papers_votes WHERE meet_up_id = ? AND user_id = ? ORDER BY vote DESC")
                .bind(meet_up_id.to_bytes().as_slice())
                .bind(user_id.to_bytes().as_slice())
                .try_map(vote_from_row)
                .fetch_all(&self.sqlite_pool)
                .await
                .map_err(|err| error_and_log!("SQLX Error: `{err}`"))?;
        Ok(votes)
    }

    async fn get_votes_for_meet_up(&self, meet_up_id: &Ulid) -> Result<Vec<Vote>, VoteError> {
        let votes = sqlx::query("SELECT * FROM meet_up_papers_votes WHERE meet_up_id = ?")
            .bind(meet_up_id.to_bytes().as_slice())
            .try_map(vote_from_row)
            .fetch_all(&self.sqlite_pool)
            .await
            .map_err(|err| error_and_log!("SQLX Error: `{err}`"))?;
        Ok(votes)
    }
}
fn vote_from_row(row: SqliteRow) -> Result<Vote, Error> {
    Ok(Vote {
        user_id: Ulid::from_bytes(
            row.try_get::<&[u8], _>("user_id")?
                .try_into()
                .map_err(|err| sqlx::Error::Decode(Box::new(err)))?,
        ),
        paper_id: Ulid::from_bytes(
            row.try_get::<&[u8], _>("paper_id")?
                .try_into()
                .map_err(|err| sqlx::Error::Decode(Box::new(err)))?,
        ),
        meet_up_id: Ulid::from_bytes(
            row.try_get::<&[u8], _>("meet_up_id")?
                .try_into()
                .map_err(|err| sqlx::Error::Decode(Box::new(err)))?,
        ),
        vote: row.try_get("vote")?,
    })
}
