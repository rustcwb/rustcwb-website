use ulid::Ulid;

use crate::{MeetUpGateway, MeetUpGoersGateway, MeetUpState};

pub async fn register_event_goer(
    meet_up_gateway: &impl MeetUpGateway,
    meet_up_goers_gateway: &impl MeetUpGoersGateway,
    user_id: &Ulid,
) -> anyhow::Result<()> {
    let meet_up = meet_up_gateway
        .get_future_meet_up()
        .await?
        .ok_or_else(|| anyhow::anyhow!("No future meetups found"))?;
    if !matches!(meet_up.state, MeetUpState::Scheduled(..)) {
        return Err(anyhow::anyhow!(
            "Invalid meet up state: {:?}",
            meet_up.state
        ));
    }
    Ok(meet_up_goers_gateway
        .register_user_to_meet_up(user_id, &meet_up.id)
        .await?)
}
