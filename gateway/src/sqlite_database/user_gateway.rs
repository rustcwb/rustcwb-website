use sqlx::{sqlite::SqliteRow, Error, Row};
use ulid::Ulid;

use domain::{AccessToken, GetUserError, LoginMethod, StoreUserError, User, UserGateway};
use shared::utc_now;

use crate::error_and_log;

use super::SqliteDatabaseGateway;

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
        .try_map(user_from_row)
        .fetch_one(&self.sqlite_pool)
        .await
        .map_err(|err| match err {
            Error::RowNotFound => GetUserError::NotFound,
            _ => GetUserError::Unknown(error_and_log!("SQLX Error: {err}")),
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
            .map_err(|err| StoreUserError::Unknown(error_and_log!("SQLX Error: {err}")))?;
        let now = utc_now();
        sqlx::query("INSERT INTO users (id, nickname, email, access_token, expires_at, login_method, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ? ,?) ON CONFLICT (id) DO UPDATE SET nickname = EXCLUDED.nickname, email = EXCLUDED.email, access_token = EXCLUDED.access_token, expires_at = EXCLUDED.expires_at, login_method = EXCLUDED.login_method, updated_at = EXCLUDED.updated_at")
            .bind(user_id)
            .bind(&user.nickname)
            .bind(&user.email)
            .bind(user.access_token.token())
            .bind(user.access_token.expire_at())
            .bind(login_method)
            .bind(now)
            .bind(now)
            .execute(&mut *transaction)
            .await
            .map_err(|err| StoreUserError::Unknown(error_and_log!("SQLX Error: {err}")))?;
        match &user.login_method {
            LoginMethod::Github {
                access_token,
                refresh_token,
            } => {
                sqlx::query("INSERT INTO github_logins (id, user_id, access_token, expires_at, refresh_token, refresh_token_expires_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT (user_id) DO UPDATE SET access_token = EXCLUDED.access_token, expires_at = EXCLUDED.expires_at, refresh_token = EXCLUDED.refresh_token, refresh_token_expires_at = EXCLUDED.refresh_token_expires_at, updated_at = EXCLUDED.updated_at")
                    .bind(Ulid::new().to_bytes().as_slice())
                    .bind(user_id)
                    .bind(access_token.token())
                    .bind(access_token.expire_at())
                    .bind(refresh_token.token())
                    .bind(refresh_token.expire_at())
                    .bind(now)
                    .bind(now)
                    .execute(&mut *transaction)
                    .await
                    .map_err(|err| StoreUserError::Unknown(error_and_log!("SQLX Error: {err}")))?;
            }
        };
        transaction
            .commit()
            .await
            .map_err(|err| StoreUserError::Unknown(error_and_log!("SQLX Error: {err}")))?;
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
        .try_map(user_from_row)
        .fetch_one(&self.sqlite_pool)
        .await
        .map_err(|err| match err {
            Error::RowNotFound => GetUserError::NotFound,
            _ => GetUserError::Unknown(error_and_log!("SQLX Error: {err}")),
        })
    }
}

fn user_from_row(row: SqliteRow) -> Result<User, Error> {
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
}
