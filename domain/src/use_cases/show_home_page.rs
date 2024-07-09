use ulid::Ulid;

use crate::{MeetUp, MeetUpGateway, MeetUpGoersGateway, MeetUpMetadata};

pub async fn show_home_page(
    meet_up_gateway: &impl MeetUpGateway,
    meet_up_goers_gateway: &impl MeetUpGoersGateway,
    user_id: Option<&Ulid>,
) -> anyhow::Result<(Option<MeetUp>, Vec<MeetUpMetadata>, bool)> {
    let future_meet_up = meet_up_gateway.get_future_meet_up().await?;
    let is_registered_user = match user_id.and_then(|user_id| {
        future_meet_up.as_ref().map(|meet_up| {
            meet_up_goers_gateway.is_user_registered_to_meet_up(user_id, &meet_up.id)
        })
    }) {
        Some(is_registered_user) => is_registered_user.await?,
        None => false,
    };
    Ok((
        future_meet_up,
        meet_up_gateway.list_past_meet_ups().await?,
        is_registered_user,
    ))
}
