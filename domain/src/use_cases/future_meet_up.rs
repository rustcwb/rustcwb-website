use anyhow::anyhow;
use chrono::NaiveDate;
use ulid::Ulid;
use url::Url;

use crate::{
    FutureMeetUp, FutureMeetUpGateway, FutureMeetUpState, NewFutureMeetUpError, VoteDecider,
    VoteGateway,
};

pub async fn create_new_future_meet_up(
    gateway: &impl FutureMeetUpGateway,
    location: String,
    date: NaiveDate,
) -> Result<FutureMeetUp, NewFutureMeetUpError> {
    gateway
        .new_future_meet_up(Ulid::new(), location, date)
        .await
}

pub async fn move_future_meet_up_to_voting(
    future_meet_up_gateway: &impl FutureMeetUpGateway,
) -> anyhow::Result<FutureMeetUp> {
    let future_meet_up = future_meet_up_gateway
        .get_future_meet_up()
        .await?
        .ok_or_else(|| anyhow::anyhow!("No future meetups found"))?;
    if future_meet_up.state != FutureMeetUpState::CallForPapers {
        return Err(anyhow::anyhow!(
            "Invalid meet up state: {:?}",
            future_meet_up.state
        ));
    }
    Ok(future_meet_up_gateway
        .update_future_meet_up_to_voting(&future_meet_up.id)
        .await?)
}

pub async fn move_future_meet_up_to_scheduled(
    future_meet_up_gateway: &impl FutureMeetUpGateway,
    vote_gateway: &impl VoteGateway,
) -> anyhow::Result<FutureMeetUp> {
    let future_meet_up = future_meet_up_gateway
        .get_future_meet_up()
        .await?
        .ok_or_else(|| anyhow::anyhow!("No future meetups found"))?;
    if future_meet_up.state != FutureMeetUpState::Voting {
        return Err(anyhow::anyhow!(
            "Invalid meet up state: {:?}",
            future_meet_up.state
        ));
    }
    let votes = vote_gateway
        .get_votes_for_meet_up(&future_meet_up.id)
        .await?;
    let paper_id = VoteDecider::new(votes)
        .decide()
        .ok_or(anyhow!("No valid paper found"))?;
    Ok(future_meet_up_gateway
        .update_future_meet_up_to_scheduled(&future_meet_up.id, &paper_id)
        .await?)
}

pub async fn move_future_meet_up_to_past_meet_up(
    future_meet_up_gateway: &impl FutureMeetUpGateway,
    link: Url,
) -> anyhow::Result<()> {
    let future_meet_up = future_meet_up_gateway
        .get_future_meet_up()
        .await?
        .ok_or_else(|| anyhow::anyhow!("No future meetups found"))?;
    if !matches!(future_meet_up.state, FutureMeetUpState::Scheduled(_)) {
        return Err(anyhow::anyhow!(
            "Invalid meet up state: {:?}",
            future_meet_up.state
        ));
    }
    future_meet_up_gateway
        .finish_future_meet_up(&future_meet_up.id, link)
        .await?;
    Ok(())
}
