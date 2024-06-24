use ulid::Ulid;

use crate::{
    FutureMeetUp, FutureMeetUpGateway, GetFutureMeetUpError, GetPastMeetUpError, PastMeetUp,
    PastMeetUpGateway, PastMeetUpMetadata,
};

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

pub async fn get_future_meet_up(
    gateway: &impl FutureMeetUpGateway,
) -> Result<Option<FutureMeetUp>, GetFutureMeetUpError> {
    gateway.get_future_meet_up().await
}
