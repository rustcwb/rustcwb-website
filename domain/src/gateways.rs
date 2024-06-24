#![allow(async_fn_in_trait)]

use chrono::NaiveDate;
use thiserror::Error;
use ulid::Ulid;
use url::Url;

use crate::{AccessToken, FutureMeetUp, Paper, PastMeetUp, PastMeetUpMetadata, User, Vote};

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
    async fn update_future_meet_up_to_voting(
        &self,
        id: &Ulid,
    ) -> Result<FutureMeetUp, UpdateFutureMeetUpError>;
    async fn update_future_meet_up_to_scheduled(
        &self,
        id: &Ulid,
        paper_id: &Ulid,
    ) -> Result<FutureMeetUp, UpdateFutureMeetUpError>;
    async fn finish_future_meet_up(
        &self,
        id: &Ulid,
        link: Url,
    ) -> Result<(), UpdateFutureMeetUpError>;
}

#[derive(Debug, Error)]
pub enum UpdateFutureMeetUpError {
    #[error("Invalid meet up state")]
    InvalidState,
    #[error("Future meet up with not found")]
    NotFound,
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
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

pub trait PaperGateway {
    async fn store_paper_with_meet_up(
        &self,
        paper: Paper,
        meet_up_id: Ulid,
        limit: u8,
    ) -> Result<(), StorePaperError>;
    async fn get_paper(&self, id: &Ulid) -> Result<Paper, GetPaperError>;
    async fn get_papers_from_user_and_meet_up(
        &self,
        user_id: &Ulid,
        meet_up_id: &Ulid,
    ) -> Result<Vec<Paper>, GetPaperError>;
    async fn get_papers_from_meet_up(&self, meet_up_id: &Ulid)
        -> Result<Vec<Paper>, GetPaperError>;
}

#[derive(Debug, Error)]
pub enum StorePaperError {
    #[error("More than limit papaers per user per meetups. Limit is `{0}`")]
    MoreThanLimitPapersPerUserPerMeetUp(u8),
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum GetPaperError {
    #[error("Paper not found with id `{0}`")]
    NotFound(Ulid),
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

pub trait VoteGateway {
    async fn store_votes(&self, votes: Vec<Vote>) -> Result<(), VoteError>;
    async fn get_votes_for_user(
        &self,
        meet_up_id: &Ulid,
        user_id: &Ulid,
    ) -> Result<Vec<Vote>, VoteError>;
    async fn get_votes_for_meet_up(&self, meet_up_id: &Ulid) -> Result<Vec<Vote>, VoteError>;
}

#[derive(Debug, Error)]
pub enum VoteError {
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}
