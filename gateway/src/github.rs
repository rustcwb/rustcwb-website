use crate::error_and_log;
use chrono::Utc;
use domain::{
    AccessToken, ExchangeCodeError, GithubGateway, RefreshTokenError, UserInfoGithubError,
};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::Deserialize;

pub struct GithubRestGateway {
    client: reqwest::Client,
    client_id: String,
    client_secret: String,
}

impl GithubRestGateway {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            client_id,
            client_secret,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct UserInfo {
    login: String,
    email: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: String,
    refresh_token_expires_in: i64,
}

impl GithubGateway for GithubRestGateway {
    async fn user_info(
        &self,
        access_token: &AccessToken,
    ) -> Result<(String, String), UserInfoGithubError> {
        let user_info = self
            .client
            .get("https://api.github.com/user")
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .header(USER_AGENT, "RustCWB/0.1.0")
            .header(AUTHORIZATION, format!("Bearer {}", access_token.token()))
            .send()
            .await
            .map_err(|err| UserInfoGithubError::Unknown(error_and_log!("Reqwest error {err}")))?
            .json::<UserInfo>()
            .await
            .map_err(|err| {
                UserInfoGithubError::Unknown(error_and_log!("Invalid json response {err}"))
            })?;
        Ok((user_info.login, user_info.email))
    }

    async fn refresh_token(
        &self,
        refresh_token: &AccessToken,
    ) -> Result<(AccessToken, AccessToken), RefreshTokenError> {
        let response = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, "RustCWB/0.1.0")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token.token()),
            ])
            .send()
            .await
            .map_err(|err| RefreshTokenError::Unknown(error_and_log!("Reqwest error {err}")))?
            .json::<AccessTokenResponse>()
            .await
            .map_err(|err| {
                RefreshTokenError::Unknown(error_and_log!("Invalid json response {err}"))
            })?;
        Ok((
            AccessToken::new(
                response.access_token,
                Utc::now() + chrono::Duration::seconds(response.expires_in),
            ),
            AccessToken::new(
                response.refresh_token,
                Utc::now() + chrono::Duration::seconds(response.refresh_token_expires_in),
            ),
        ))
    }

    async fn exchange_code(
        &self,
        code: &str,
    ) -> Result<(AccessToken, AccessToken), ExchangeCodeError> {
        let response = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, "RustCWB/0.1.0")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", code),
            ])
            .send()
            .await
            .map_err(|err| ExchangeCodeError::Unknown(error_and_log!("Reqwest error {err}")))?
            .text()
            .await
            .map_err(|err| ExchangeCodeError::Unknown(error_and_log!("Invalid response {err}")))?;
        let response: AccessTokenResponse = serde_json::from_str(&response).map_err(|err| {
            ExchangeCodeError::Unknown(error_and_log!("Invalid json response {err}. {response}"))
        })?;
        Ok((
            AccessToken::new(
                response.access_token,
                Utc::now() + chrono::Duration::seconds(response.expires_in),
            ),
            AccessToken::new(
                response.refresh_token,
                Utc::now() + chrono::Duration::seconds(response.refresh_token_expires_in),
            ),
        ))
    }
}
