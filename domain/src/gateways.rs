#![allow(async_fn_in_trait)]

use chrono::NaiveDate;
use thiserror::Error;
use ulid::Ulid;
use url::Url;

use crate::{AccessToken, MeetUp, MeetUpMetadata, Paper, User, Vote};

#[derive(Debug, Error)]
pub enum ListPastMeetUpsError {
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum GetMeetUpError {
    #[error("Meet up with id `{0}` not found")]
    NotFound(Ulid),
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

pub trait MeetUpGateway {
    async fn get_future_meet_up(&self) -> Result<Option<MeetUp>, GetFutureMeetUpError>;
    async fn list_past_meet_ups(&self) -> Result<Vec<MeetUpMetadata>, ListPastMeetUpsError>;
    async fn get_meet_up(&self, id: &Ulid) -> Result<MeetUp, GetMeetUpError>;
    async fn get_meet_up_metadata(
        &self,
        id: Ulid,
    ) -> anyhow::Result<MeetUpMetadata, GetMeetUpError>;
    async fn new_meet_up(
        &self,
        id: Ulid,
        location: String,
        date: NaiveDate,
    ) -> Result<MeetUp, NewMeetUpError>;
    async fn update_meet_up_to_voting(&self, id: &Ulid) -> Result<MeetUp, UpdateMeetUpError>;
    async fn update_meet_up_to_scheduled(
        &self,
        id: &Ulid,
        paper_id: &Ulid,
    ) -> Result<MeetUp, UpdateMeetUpError>;
    async fn finish_meet_up(&self, id: &Ulid, link: Url) -> Result<(), UpdateMeetUpError>;
}

#[derive(Debug, Error)]
pub enum UpdateMeetUpError {
    #[error("Invalid meet up state")]
    InvalidState,
    #[error("Meet up with id `{0}` not found")]
    NotFound(Ulid),
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum GetFutureMeetUpError {
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum NewMeetUpError {
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
        paper: &Paper,
        meet_up_id: &Ulid,
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

pub trait MeetUpGoersGateway {
    async fn register_user_to_meet_up(
        &self,
        user_id: &Ulid,
        meet_up_id: &Ulid,
    ) -> Result<(), RegisterUserError>;
    async fn is_user_registered_to_meet_up(
        &self,
        user_id: &Ulid,
        meet_up_id: &Ulid,
    ) -> Result<bool, RegisterUserError>;

    async fn get_number_attendees_from_meet_up(
        &self,
        meet_up_id: &Ulid,
    ) -> Result<usize, GetAttendeesError>;
}

#[derive(Debug, Error)]
pub enum RegisterUserError {
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum GetAttendeesError {
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}
