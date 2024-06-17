#![allow(async_fn_in_trait)]

use chrono::NaiveDate;
use thiserror::Error;
use ulid::Ulid;

use crate::{AccessToken, FutureMeetUp, PastMeetUp, PastMeetUpMetadata, User};

pub trait PastMeetUpGateway {
    async fn list_past_meet_ups(&self) -> Result<Vec<PastMeetUpMetadata>, ListPastMeetUpsError>;
    async fn get_past_meet_up(&self, id: Ulid) -> Result<PastMeetUp, GetPastMeetUpError>;
    async fn get_past_meet_up_metadata(
        &self,
        id: Ulid,
    ) -> anyhow::Result<PastMeetUpMetadata, GetPastMeetUpError>;
}

#[derive(Debug, Error)]
pub enum ListPastMeetUpsError {
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum GetPastMeetUpError {
    #[error("Past meet up with id `{0}` not found")]
    NotFound(Ulid),
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

pub trait FutureMeetUpGateway {
    async fn get_future_meet_up(&self) -> Result<Option<FutureMeetUp>, GetFutureMeetUpError>;
    async fn new_future_meet_up(
        &self,
        id: Ulid,
        location: String,
        date: NaiveDate,
    ) -> Result<FutureMeetUp, NewFutureMeetUpError>;
}

#[derive(Debug, Error)]
pub enum GetFutureMeetUpError {
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum NewFutureMeetUpError {
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

pub trait UserGateway {
    async fn get_user_with_token(&self, access_token: &str) -> Result<User, GetUserError>;
    async fn get_user_with_email(&self, email: &str) -> Result<User, GetUserError>;
    async fn store_user(&self, user: User) -> Result<User, StoreUserError>;
}

#[derive(Debug, Error)]
pub enum GetUserError {
    #[error("User not found")]
    NotFound,
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum StoreUserError {
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

pub trait GithubGateway {
    async fn user_info(
        &self,
        access_token: &AccessToken,
    ) -> Result<(String, String), UserInfoGithubError>;
    async fn refresh_token(
        &self,
        refresh_token: &AccessToken,
    ) -> Result<(AccessToken, AccessToken), RefreshTokenError>;
    async fn exchange_code(
        &self,
        code: &str,
    ) -> Result<(AccessToken, AccessToken), ExchangeCodeError>;
}

#[derive(Debug, Error)]
pub enum UserInfoGithubError {
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum RefreshTokenError {
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum ExchangeCodeError {
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}
