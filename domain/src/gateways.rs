#![allow(async_fn_in_trait)]
use thiserror::Error;
use ulid::Ulid;

use crate::{PastMeetUp, PastMeetUpMetadata};

pub trait PastMeetUpGateway {
    async fn list_past_meet_ups(&self) -> Result<Vec<PastMeetUpMetadata>, ListPastMeetUpsError>;
    async fn get_past_meet_up(&self, id: Ulid) -> Result<PastMeetUp, GetPastMeetUpError>;
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
