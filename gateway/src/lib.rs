use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use sqlx::{sqlite::SqliteRow, Error, Row, SqlitePool};
use ulid::Ulid;
use url::Url;

pub mod github;

use domain::{
    AccessToken, FutureMeetUp, FutureMeetUpGateway, FutureMeetUpState, GetFutureMeetUpError,
    GetPastMeetUpError, GetUserError, ListPastMeetUpsError, LoginMethod, NewFutureMeetUpError,
    PastMeetUp, PastMeetUpGateway, PastMeetUpMetadata, StoreUserError, User, UserGateway,
};

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

    async fn new_future_meet_up(
        &self,
        id: Ulid,
        location: String,
        date: NaiveDate,
    ) -> std::result::Result<FutureMeetUp, NewFutureMeetUpError> {
        sqlx::query("INSERT INTO future_meet_ups (id, state, location, date) VALUES (?, ?, ?, ?)")
            .bind(id.to_bytes().as_slice())
            .bind(0)
            .bind(&location)
            .bind(&date)
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

impl UserGateway for SqliteDatabaseGateway {
    async fn get_user_with_token(&self, access_token: &str) -> Result<User, GetUserError> {
        sqlx::query(
            r#"
            SELECT u.id user_id,
                u.nickname,
                u.email,
                u.access_token,
                u.expires_at,
                u.login_method,
                gl.access_token github_access_token,
                gl.expires_at github_expires_at,
                gl.refresh_token,
                gl.refresh_token_expires_at
            FROM users u
            JOIN github_logins gl ON u.id = gl.user_id
            WHERE u.access_token = ?"#,
        )
        .bind(access_token)
        .try_map(|row: SqliteRow| {
            Ok(User::new(
                Ulid::from_bytes(
                    row.try_get::<&[u8], _>("user_id")?
                        .try_into()
                        .map_err(|err| Error::Decode(Box::new(err)))?,
                ),
                row.get("nickname"),
                row.get("email"),
                AccessToken::new(row.get("access_token"), row.get("expires_at")),
                match row.get("login_method") {
                    0 => LoginMethod::Github {
                        access_token: AccessToken::new(
                            row.get("github_access_token"),
                            row.get("github_expires_at"),
                        ),
                        refresh_token: AccessToken::new(
                            row.get("refresh_token"),
                            row.get("refresh_token_expires_at"),
                        ),
                    },
                    _ => return Err(Error::Decode("Unknown login method".into())),
                },
            ))
        })
        .fetch_one(&self.sqlite_pool)
        .await
        .map_err(|err| match err {
            Error::RowNotFound => GetUserError::NotFound,
            _ => GetUserError::Unknown(anyhow!("SQLX Error: {err}")),
        })
    }

    async fn store_user(&self, user: User) -> Result<User, StoreUserError> {
        let id = user.id.to_bytes();
        let user_id = id.as_slice();
        let login_method = match user.login_method {
            LoginMethod::Github { .. } => 0,
        };
        let mut transaction = self
            .sqlite_pool
            .begin()
            .await
            .map_err(|err| StoreUserError::Unknown(anyhow!("SQLX Error: {err}")))?;
        sqlx::query("INSERT INTO users (id, nickname, email, access_token, expires_at, login_method) VALUES (?, ?, ?, ?, ?, ?) ON CONFLICT (id) DO UPDATE SET nickname = EXCLUDED.nickname, email = EXCLUDED.email, access_token = EXCLUDED.access_token, expires_at = EXCLUDED.expires_at, login_method = EXCLUDED.login_method")
            .bind(&user_id)
            .bind(&user.nickname)
            .bind(&user.email)
            .bind(user.access_token.token())
            .bind(user.access_token.expire_at())
            .bind(login_method)
            .execute(&mut *transaction)
            .await
            .map_err(|err| StoreUserError::Unknown(anyhow!("SQLX Error: {err}")))?;
        match &user.login_method {
            LoginMethod::Github {
                access_token,
                refresh_token,
            } => {
                sqlx::query("INSERT INTO github_logins (id, user_id, access_token, expires_at, refresh_token, refresh_token_expires_at) VALUES (?, ?, ?, ?, ?, ?) ON CONFLICT (user_id) DO UPDATE SET access_token = EXCLUDED.access_token, expires_at = EXCLUDED.expires_at, refresh_token = EXCLUDED.refresh_token, refresh_token_expires_at = EXCLUDED.refresh_token_expires_at")
                    .bind(Ulid::new().to_bytes().as_slice())
                    .bind(&user_id)
                    .bind(access_token.token())
                    .bind(access_token.expire_at())
                    .bind(refresh_token.token())
                    .bind(refresh_token.expire_at())
                    .execute(&mut *transaction)
                    .await
                    .map_err(|err| StoreUserError::Unknown(anyhow!("SQLX Error: {err}")))?;
            }
        };
        transaction
            .commit()
            .await
            .map_err(|err| StoreUserError::Unknown(anyhow!("SQLX Error: {err}")))?;
        Ok(user)
    }

    async fn get_user_with_email(&self, email: &str) -> Result<User, GetUserError> {
        sqlx::query(
            r#"
            SELECT u.id user_id,
                u.nickname,
                u.email,
                u.access_token,
                u.expires_at,
                u.login_method,
                gl.access_token github_access_token,
                gl.expires_at github_expires_at,
                gl.refresh_token,
                gl.refresh_token_expires_at
            FROM users u
            JOIN github_logins gl ON u.id = gl.user_id
            WHERE u.email = ?"#,
        )
        .bind(email)
        .try_map(|row: SqliteRow| {
            dbg!(row.columns());
            Ok(User::new(
                Ulid::from_bytes(
                    row.try_get::<&[u8], _>("user_id")?
                        .try_into()
                        .map_err(|err| Error::Decode(Box::new(err)))?,
                ),
                row.get("nickname"),
                row.get("email"),
                AccessToken::new(row.get("access_token"), row.get("expires_at")),
                match row.get("login_method") {
                    0 => LoginMethod::Github {
                        access_token: AccessToken::new(
                            row.get("github_access_token"),
                            row.get("github_expires_at"),
                        ),
                        refresh_token: AccessToken::new(
                            row.get("refresh_token"),
                            row.get("refresh_token_expires_at"),
                        ),
                    },
                    _ => return Err(Error::Decode("Unknown login method".into())),
                },
            ))
        })
        .fetch_one(&self.sqlite_pool)
        .await
        .map_err(|err| match err {
            Error::RowNotFound => GetUserError::NotFound,
            _ => GetUserError::Unknown(anyhow!("SQLX Error: {err}")),
        })
    }
}
