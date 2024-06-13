use ulid::Ulid;

use crate::{
    gateways::{ListPastMeetUpsError, PastMeetUpGateway},
    GetPastMeetUpError, PastMeetUp, PastMeetUpMetadata,
};

pub async fn list_past_meet_ups(
    gateway: &impl PastMeetUpGateway,
) -> Result<Vec<PastMeetUpMetadata>, ListPastMeetUpsError> {
    gateway.list_past_meet_ups().await
}

pub async fn get_past_meet_ups(
    gateway: &impl PastMeetUpGateway,
    id: Ulid,
) -> Result<PastMeetUp, GetPastMeetUpError> {
    gateway.get_past_meet_up(id).await
}
