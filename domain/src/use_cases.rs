use std::collections::HashMap;

use anyhow::{anyhow, bail};
use chrono::NaiveDate;
use thiserror::Error;
use ulid::Ulid;

use crate::{AccessToken, FutureMeetUp, FutureMeetUpGateway, FutureMeetUpState, gateways::PastMeetUpGateway, GetFutureMeetUpError, GetPaperError, GetPastMeetUpError, GetUserError, GithubGateway, LoginMethod, NewFutureMeetUpError, Paper, PaperGateway, PastMeetUp, PastMeetUpMetadata, StorePaperError, User, UserGateway, Vote, VoteGateway};

const MAX_PAPERS_PER_USER_PER_MEET_UP: u8 = 2;

pub async fn show_home_page(
    past_meet_up_gateway: &impl PastMeetUpGateway,
    future_meet_up_gateway: &impl FutureMeetUpGateway,
) -> anyhow::Result<(Option<FutureMeetUp>, Vec<PastMeetUpMetadata>)> {
    Ok((
        future_meet_up_gateway.get_future_meet_up().await?,
        past_meet_up_gateway.list_past_meet_ups().await?,
    ))
}

pub async fn show_admin_page(
    future_meet_up_gateway: &impl FutureMeetUpGateway,
    papers_gateway: &impl PaperGateway,
) -> anyhow::Result<(Option<FutureMeetUp>, usize)> {
    let future_meet_up = future_meet_up_gateway.get_future_meet_up().await?;
    let n_papers = match &future_meet_up {
        None => 0,
        Some(future_meet_up) => papers_gateway
            .get_papers_from_meet_up(&future_meet_up.id)
            .await?
            .len(),
    };
    Ok((future_meet_up, n_papers))
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

pub async fn get_future_meet_up(
    gateway: &impl FutureMeetUpGateway,
) -> Result<Option<FutureMeetUp>, GetFutureMeetUpError> {
    gateway.get_future_meet_up().await
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

pub async fn create_new_future_meet_up(
    gateway: &impl FutureMeetUpGateway,
    location: String,
    date: NaiveDate,
) -> Result<FutureMeetUp, NewFutureMeetUpError> {
    gateway
        .new_future_meet_up(Ulid::new(), location, date)
        .await
}

pub async fn login_with_access_token(
    user_gateway: &impl UserGateway,
    github_gateway: &impl GithubGateway,
    access_token: &str,
) -> anyhow::Result<User> {
    let user = user_gateway.get_user_with_token(access_token).await?;
    if !user.access_token.is_expired() {
        return Ok(user);
    }
    match user.login_method {
        LoginMethod::Github {
            mut access_token,
            mut refresh_token,
        } => {
            if access_token.is_expired() {
                if refresh_token.is_expired() {
                    return Err(anyhow::anyhow!("Refresh token is expired"));
                }
                let (new_access_token, new_refresh_token) =
                    github_gateway.refresh_token(&refresh_token).await?;
                access_token = new_access_token;
                refresh_token = new_refresh_token;
            }
            let (nickname, email) = github_gateway.user_info(&access_token).await?;
            let user = user_gateway
                .store_user(User {
                    id: user.id,
                    access_token: AccessToken::generate_new(),
                    login_method: LoginMethod::Github {
                        access_token,
                        refresh_token,
                    },
                    nickname,
                    email,
                })
                .await?;
            Ok(user)
        }
    }
}

pub async fn login_with_github_code(
    user_gateway: &impl UserGateway,
    github_gateway: &impl GithubGateway,
    code: String,
) -> anyhow::Result<User> {
    let (access_token, refresh_token) = github_gateway.exchange_code(&code).await?;
    let (nickname, email) = github_gateway.user_info(&access_token).await?;
    // This code can have problems with concurrency. But this should be very unlikely
    let user = match user_gateway.get_user_with_email(&email).await {
        Ok(user) => {
            user_gateway
                .store_user(User {
                    id: user.id,
                    access_token: AccessToken::generate_new(),
                    login_method: LoginMethod::Github {
                        access_token,
                        refresh_token,
                    },
                    nickname,
                    email,
                })
                .await?
        }
        Err(GetUserError::NotFound) => {
            user_gateway
                .store_user(User {
                    id: Ulid::new(),
                    access_token: AccessToken::generate_new(),
                    login_method: LoginMethod::Github {
                        access_token,
                        refresh_token,
                    },
                    nickname,
                    email,
                })
                .await?
        }
        Err(err) => bail!("Error: {:?}", err),
    };
    Ok(user)
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
    let votes = vote_gateway
        .get_votes_for_user(&future_meet_up.id, user_id)
        .await?;
    let papers = papers_gateway
        .get_papers_from_meet_up(&future_meet_up.id)
        .await?;
    if votes.is_empty() {
        vote_gateway.store_votes(
            papers.iter().enumerate().map(|(vote, paper)| Vote {
                user_id: user_id.clone(),
                paper_id: paper.id.clone(),
                meet_up_id: future_meet_up.id.clone(),
                vote: vote as u32,
            }).collect()
        ).await?;
        return Ok((future_meet_up, papers));
    }
    let mut papers = papers
        .into_iter()
        .map(|paper| (paper.id.clone(), paper))
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
            user_id: user_id.clone(),
            paper_id,
            meet_up_id: future_meet_up.id.clone(),
            vote: vote as u32,
        })
        .collect();
    vote_gateway.store_votes(votes).await?;
    Ok(())
}

pub async fn get_paper(
    paper_gateway: &impl PaperGateway,
    id: &Ulid,
) -> Result<Paper, GetPaperError> {
    paper_gateway.get_paper(id).await
}