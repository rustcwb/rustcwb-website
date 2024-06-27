use crate::{MeetUp, MeetUpGateway, MeetUpMetadata};

pub async fn show_home_page(
    meet_up_gateway: &impl MeetUpGateway,
) -> anyhow::Result<(Option<MeetUp>, Vec<MeetUpMetadata>)> {
    Ok((
        meet_up_gateway.get_future_meet_up().await?,
        meet_up_gateway.list_past_meet_ups().await?,
    ))
}
