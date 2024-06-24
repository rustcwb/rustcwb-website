use std::collections::HashMap;

use anyhow::anyhow;
use ulid::Ulid;

use crate::{
    FutureMeetUp, FutureMeetUpGateway, FutureMeetUpState, Paper, PaperGateway, Vote, VoteGateway,
};

pub async fn show_voting(
    future_meet_up_gateway: &impl FutureMeetUpGateway,
    papers_gateway: &impl PaperGateway,
    vote_gateway: &impl VoteGateway,
    user_id: &Ulid,
) -> anyhow::Result<(FutureMeetUp, Vec<Paper>)> {
    let future_meet_up = future_meet_up_gateway
        .get_future_meet_up()
        .await?
        .ok_or(anyhow!("No meet up found"))?;
    if future_meet_up.state != FutureMeetUpState::Voting {
        return Err(anyhow!("Invalid meet up state: {:?}", future_meet_up.state));
    }
    let votes = vote_gateway
        .get_votes_for_user(&future_meet_up.id, user_id)
        .await?;
    let papers = papers_gateway
        .get_papers_from_meet_up(&future_meet_up.id)
        .await?;
    if votes.is_empty() {
        vote_gateway
            .store_votes(
                papers
                    .iter()
                    .enumerate()
                    .map(|(vote, paper)| Vote {
                        user_id: *user_id,
                        paper_id: paper.id,
                        meet_up_id: future_meet_up.id,
                        vote: vote as u32,
                    })
                    .collect(),
            )
            .await?;
        return Ok((future_meet_up, papers));
    }
    let mut papers = papers
        .into_iter()
        .map(|paper| (paper.id, paper))
        .collect::<HashMap<Ulid, Paper>>();
    let papers = votes
        .into_iter()
        .map(|vote| {
            papers
                .remove(&vote.paper_id)
                .ok_or(anyhow!("Vote for invalid paper '{}'", vote.paper_id))
        })
        .collect::<anyhow::Result<Vec<Paper>>>()?;
    Ok((future_meet_up, papers))
}

pub async fn store_votes(
    future_meet_up_gateway: &impl FutureMeetUpGateway,
    vote_gateway: &impl VoteGateway,
    user_id: &Ulid,
    papers: Vec<Ulid>,
) -> anyhow::Result<()> {
    let future_meet_up = future_meet_up_gateway
        .get_future_meet_up()
        .await?
        .ok_or(anyhow!("No meet up found"))?;
    let votes = papers
        .into_iter()
        .enumerate()
        .map(|(vote, paper_id)| Vote {
            user_id: *user_id,
            paper_id,
            meet_up_id: future_meet_up.id,
            vote: vote as u32,
        })
        .collect();
    vote_gateway.store_votes(votes).await?;
    Ok(())
}
