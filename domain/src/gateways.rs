#![allow(async_fn_in_trait)]

use chrono::NaiveDate;
use thiserror::Error;
use ulid::Ulid;

use crate::{FutureMeetUp, PastMeetUp, PastMeetUpMetadata};

pub trait PastMeetUpGateway {
    async fn list_past_meet_ups(&self) -> Result<Vec<PastMeetUpMetadata>, ListPastMeetUpsError>;
    async fn get_past_meet_up(&self, id: Ulid) -> Result<PastMeetUp, GetPastMeetUpError>;
    async fn get_past_meet_up_metadata(&self, id: Ulid) -> anyhow::Result<PastMeetUpMetadata, GetPastMeetUpError>;
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
    async fn new_future_meet_up(&self, id: Ulid, location: String, date: NaiveDate) -> Result<FutureMeetUp, NewFutureMeetUpError>;
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
