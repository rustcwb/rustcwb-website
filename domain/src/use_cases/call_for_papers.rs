use thiserror::Error;
use ulid::Ulid;

use crate::{
    FutureMeetUp, FutureMeetUpGateway, FutureMeetUpState, GetPaperError, Paper, PaperGateway,
    StorePaperError, User,
};

const MAX_PAPERS_PER_USER_PER_MEET_UP: u8 = 2;

pub async fn show_call_for_papers(
    paper_gateway: &impl PaperGateway,
    future_meet_up_gateway: &impl FutureMeetUpGateway,
    user: &User,
) -> anyhow::Result<(FutureMeetUp, Vec<Paper>, bool)> {
    let future_meet_up = future_meet_up_gateway
        .get_future_meet_up()
        .await?
        .ok_or_else(|| anyhow::anyhow!("No future meetups found"))?;
    let papers = paper_gateway
        .get_papers_from_user_and_meet_up(&user.id, &future_meet_up.id)
        .await?;
    let is_limit_of_papers_sent = papers.len() as u8 >= MAX_PAPERS_PER_USER_PER_MEET_UP;
    Ok((future_meet_up, papers, is_limit_of_papers_sent))
}

pub async fn submit_paper(
    paper_gateway: &impl PaperGateway,
    future_meet_up_gateway: &impl FutureMeetUpGateway,
    paper: Paper,
) -> Result<(), SubmitPaperError> {
    let future_meet_up = future_meet_up_gateway
        .get_future_meet_up()
        .await
        .map_err(|err| SubmitPaperError::Unknown(err.into()))?
        .ok_or(SubmitPaperError::NoFutureMeetUpFound)?;
    // We can have a concurrency problem here, but we are not handling it for now.
    if future_meet_up.state != FutureMeetUpState::CallForPapers {
        return Err(SubmitPaperError::InvalidMeetUpState(future_meet_up.state));
    }

    paper_gateway
        .store_paper_with_meet_up(paper, future_meet_up.id, MAX_PAPERS_PER_USER_PER_MEET_UP)
        .await
        .map_err(|err| match err {
            StorePaperError::MoreThanLimitPapersPerUserPerMeetUp(limit) => {
                SubmitPaperError::MoreThanLimitPapersPerUserPerMeetUp(limit)
            }
            _ => SubmitPaperError::Unknown(err.into()),
        })
}

pub async fn get_paper(
    paper_gateway: &impl PaperGateway,
    id: &Ulid,
) -> Result<Paper, GetPaperError> {
    paper_gateway.get_paper(id).await
}

#[derive(Debug, Error)]
pub enum SubmitPaperError {
    #[error("Invalid meet up state: `{0}`")]
    InvalidMeetUpState(FutureMeetUpState),
    #[error("No future meetups found")]
    NoFutureMeetUpFound,
    #[error("More than limit papaers per user per meetups. Limit is `{0}`")]
    MoreThanLimitPapersPerUserPerMeetUp(u8),
    #[error("Unknown error: `{0}`")]
    Unknown(#[from] anyhow::Error),
}
