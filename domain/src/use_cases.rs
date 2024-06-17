use chrono::NaiveDate;
use ulid::Ulid;

use crate::{FutureMeetUp, FutureMeetUpGateway, gateways::PastMeetUpGateway, GetPastMeetUpError, NewFutureMeetUpError, PastMeetUp, PastMeetUpMetadata};

pub async fn show_home_page(
    past_meet_up_gateway: &impl PastMeetUpGateway,
    future_meet_up_gateway: &impl FutureMeetUpGateway,
) -> anyhow::Result<(Option<FutureMeetUp>, Vec<PastMeetUpMetadata>)> {
    Ok((
        future_meet_up_gateway.get_future_meet_up().await?,
        past_meet_up_gateway.list_past_meet_ups().await?,
    ))
}

pub async fn get_past_meet_up(
    gateway: &impl PastMeetUpGateway,
    id: Ulid,
) -> Result<PastMeetUp, GetPastMeetUpError> {
    gateway.get_past_meet_up(id).await
}

pub async fn get_past_meet_up_metadata(
    gateway: &impl PastMeetUpGateway,
    id: Ulid,
) -> Result<PastMeetUpMetadata, GetPastMeetUpError> {
    gateway.get_past_meet_up_metadata(id).await
}

pub async fn create_new_future_meet_up(
    gateway: &impl FutureMeetUpGateway,
    location: String,
    date: NaiveDate,
) -> Result<FutureMeetUp, NewFutureMeetUpError> {
    gateway.new_future_meet_up(Ulid::new(), location, date).await
}
