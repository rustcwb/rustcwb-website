use crate::{FutureMeetUp, FutureMeetUpGateway, PastMeetUpGateway, PastMeetUpMetadata};

pub async fn show_home_page(
    past_meet_up_gateway: &impl PastMeetUpGateway,
    future_meet_up_gateway: &impl FutureMeetUpGateway,
) -> anyhow::Result<(Option<FutureMeetUp>, Vec<PastMeetUpMetadata>)> {
    Ok((
        future_meet_up_gateway.get_future_meet_up().await?,
        past_meet_up_gateway.list_past_meet_ups().await?,
    ))
}
