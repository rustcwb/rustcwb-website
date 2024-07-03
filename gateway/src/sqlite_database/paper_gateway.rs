use sqlx::{sqlite::SqliteRow, Error, Row};
use ulid::Ulid;

use domain::{GetPaperError, Paper, PaperGateway, StorePaperError};

use crate::{error_and_log, SqliteDatabaseGateway};

impl PaperGateway for SqliteDatabaseGateway {
    async fn store_paper_with_meet_up(
        &self,
        paper: &Paper,
        meet_up_id: &Ulid,
        limit: u8,
    ) -> Result<(), StorePaperError> {
        let mut transaction = self
            .sqlite_pool
            .begin()
            .await
            .map_err(|err| error_and_log!("SQLX Error: {err}"))?;
        sqlx::query("INSERT INTO papers (id, title, description, speaker, email, user_id) VALUES (?, ?, ?, ?, ?, ?);")
            .bind(paper.id.to_bytes().as_slice())
            .bind(&paper.title)
            .bind(&paper.description)
            .bind(&paper.speaker)
            .bind(&paper.email)
            .bind(paper.user_id.to_bytes().as_slice())
            .execute(&mut *transaction).await.map_err(|err| error_and_log!("SQLX Error: {err}"))?;
        sqlx::query("INSERT INTO meet_up_papers (meet_up_id, paper_id) VALUES (?, ?);")
            .bind(meet_up_id.to_bytes().as_slice())
            .bind(paper.id.to_bytes().as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|err| error_and_log!("SQLX Error: {err}"))?;
        let n_papers_for_user = sqlx::query("SELECT count(1) count FROM papers p JOIN meet_up_papers mup ON mup.paper_id = p.id WHERE mup.meet_up_id = ? AND p.user_id = ?;")
            .bind(meet_up_id.to_bytes().as_slice())
            .bind(paper.user_id.to_bytes().as_slice())
            .try_map(|row: SqliteRow| {
                Ok(row.get::<u8, _>("count"))
            })
            .fetch_one(&mut *transaction)
            .await
            .map_err(|err| error_and_log!("SQLX Error: {err}"))?;
        if n_papers_for_user > limit {
            transaction
                .rollback()
                .await
                .map_err(|err| error_and_log!("SQLX Error: {err}"))?;
            return Err(StorePaperError::MoreThanLimitPapersPerUserPerMeetUp(
                n_papers_for_user,
            ));
        }
        transaction
            .commit()
            .await
            .map_err(|err| error_and_log!("SQLX Error: {err}"))?;

        Ok(())
    }

    async fn get_paper(&self, id: &Ulid) -> Result<Paper, GetPaperError> {
        let result = sqlx::query("SELECT * FROM papers WHERE id = ?")
            .bind(id.to_bytes().as_slice())
            .try_map(paper_from_row)
            .fetch_one(&self.sqlite_pool)
            .await;
        match result {
            Ok(paper) => Ok(paper),
            Err(sqlx::Error::RowNotFound) => Err(GetPaperError::NotFound(*id)),
            Err(err) => Err(GetPaperError::Unknown(error_and_log!("SQLX Error: {err}"))),
        }
    }

    async fn get_papers_from_user_and_meet_up(
        &self,
        user_id: &Ulid,
        meet_up_id: &Ulid,
    ) -> Result<Vec<Paper>, GetPaperError> {
        let result = sqlx::query("SELECT * FROM papers p JOIN meet_up_papers mup ON p.id = mup.paper_id WHERE mup.meet_up_id = ? AND p.user_id = ?")
            .bind(meet_up_id.to_bytes().as_slice())
            .bind(user_id.to_bytes().as_slice())
            .try_map(paper_from_row)
            .fetch_all(&self.sqlite_pool)
            .await
            .map_err(|err| GetPaperError::Unknown(error_and_log!("SQLX Error: {err}")))?;
        Ok(result)
    }

    async fn get_papers_from_meet_up(
        &self,
        meet_up_id: &Ulid,
    ) -> Result<Vec<Paper>, GetPaperError> {
        let result = sqlx::query("SELECT * FROM papers p JOIN meet_up_papers mup ON p.id = mup.paper_id WHERE mup.meet_up_id = ?")
            .bind(meet_up_id.to_bytes().as_slice())
            .try_map(paper_from_row)
            .fetch_all(&self.sqlite_pool)
            .await
            .map_err(|err| GetPaperError::Unknown(error_and_log!("SQLX Error: {err}")))?;
        Ok(result)
    }
}

fn paper_from_row(row: SqliteRow) -> Result<Paper, Error> {
    Ok(Paper {
        id: Ulid::from_bytes(
            row.try_get::<&[u8], _>("id")?
                .try_into()
                .map_err(|err| sqlx::Error::Decode(Box::new(err)))?,
        ),
        title: row.get("title"),
        description: row.get("description"),
        speaker: row.get("speaker"),
        email: row.get("email"),
        user_id: Ulid::from_bytes(
            row.try_get::<&[u8], _>("user_id")?
                .try_into()
                .map_err(|err| sqlx::Error::Decode(Box::new(err)))?,
        ),
    })
}
