use anyhow::anyhow;
use chrono::NaiveDate;
use thiserror::Error;
use ulid::Ulid;
use url::Url;

use crate::{
    GetFutureMeetUpError, GetMeetUpError, Location, MeetUp, MeetUpGateway, MeetUpMetadata,
    MeetUpState, NewMeetUpError, VoteDecider, VoteGateway,
};

pub async fn create_new_meet_up(
    gateway: &impl MeetUpGateway,
    location: Location,
    date: NaiveDate,
) -> Result<MeetUp, NewMeetUpError> {
    gateway.new_meet_up(Ulid::new(), location, date).await
}

pub async fn move_future_meet_up_to_voting(gateway: &impl MeetUpGateway) -> anyhow::Result<MeetUp> {
    let meet_up = gateway
        .get_future_meet_up()
        .await?
        .ok_or_else(|| anyhow::anyhow!("No future meetups found"))?;
    if meet_up.state != MeetUpState::CallForPapers {
        return Err(anyhow::anyhow!(
            "Invalid meet up state: {:?}",
            meet_up.state
        ));
    }
    Ok(gateway.update_meet_up_to_voting(&meet_up.id).await?)
}

pub async fn move_future_meet_up_to_scheduled(
    gateway: &impl MeetUpGateway,
    vote_gateway: &impl VoteGateway,
) -> anyhow::Result<MeetUp> {
    let meet_up = gateway
        .get_future_meet_up()
        .await?
        .ok_or_else(|| anyhow::anyhow!("No future meetups found"))?;
    if meet_up.state != MeetUpState::Voting {
        return Err(anyhow::anyhow!(
            "Invalid meet up state: {:?}",
            meet_up.state
        ));
    }
    let votes = vote_gateway.get_votes_for_meet_up(&meet_up.id).await?;
    let paper_id = VoteDecider::new(votes)
        .decide()
        .ok_or(anyhow!("No valid paper found"))?;
    Ok(gateway
        .update_meet_up_to_scheduled(&meet_up.id, &paper_id)
        .await?)
}

pub async fn move_future_meet_up_to_done(
    gateway: &impl MeetUpGateway,
    link: Url,
) -> anyhow::Result<()> {
    let meet_up = gateway
        .get_future_meet_up()
        .await?
        .ok_or_else(|| anyhow::anyhow!("No future meetups found"))?;
    if !matches!(meet_up.state, MeetUpState::Scheduled(_)) {
        return Err(anyhow::anyhow!(
            "Invalid meet up state: {:?}",
            meet_up.state
        ));
    }
    gateway.finish_meet_up(&meet_up.id, link).await?;
    Ok(())
}

pub async fn get_meet_up(
    gateway: &impl MeetUpGateway,
    id: Ulid,
) -> Result<MeetUp, GetPastMeetUpError> {
    Ok(gateway.get_meet_up(&id).await?)
}

pub async fn get_meet_up_metadata(
    gateway: &impl MeetUpGateway,
    id: Ulid,
) -> Result<MeetUpMetadata, GetPastMeetUpError> {
    Ok(gateway.get_meet_up_metadata(id).await?)
}

pub async fn get_future_meet_up(
    gateway: &impl MeetUpGateway,
) -> Result<Option<MeetUp>, GetFutureMeetUpError> {
    gateway.get_future_meet_up().await
}

#[derive(Debug, Error)]
pub enum GetPastMeetUpError {
    #[error("Meet up with `{0} not found")]
    NotFound(Ulid),
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}

impl From<GetMeetUpError> for GetPastMeetUpError {
    fn from(err: GetMeetUpError) -> Self {
        match err {
            GetMeetUpError::NotFound(id) => GetPastMeetUpError::NotFound(id),
            err => GetPastMeetUpError::Unknown(err.into()),
        }
    }
}
