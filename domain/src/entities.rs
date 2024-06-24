use std::{collections::HashMap, fmt::Debug};

use chrono::{DateTime, NaiveDate, Utc};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastMeetUp {
    pub id: Ulid,
    pub paper: Paper,
    pub date: NaiveDate,
    pub link: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastMeetUpMetadata {
    id: Ulid,
    title: String,
    date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FutureMeetUp {
    pub id: Ulid,
    pub state: FutureMeetUpState,
    pub location: String,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FutureMeetUpState {
    CallForPapers,
    Voting,
    Scheduled(Paper),
}

impl std::fmt::Display for FutureMeetUpState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FutureMeetUpState::CallForPapers => write!(f, "CallForPapers"),
            FutureMeetUpState::Voting => write!(f, "Voting"),
            FutureMeetUpState::Scheduled { .. } => write!(f, "Scheduled"),
        }
    }
}

impl FutureMeetUp {
    pub fn new(id: Ulid, state: FutureMeetUpState, location: String, date: NaiveDate) -> Self {
        Self {
            id,
            state,
            location,
            date,
        }
    }
}

impl PastMeetUp {
    pub fn new(id: Ulid, paper: Paper, date: NaiveDate, link: Url) -> Self {
        Self {
            id,
            paper,
            date,
            link,
        }
    }
}

impl PastMeetUpMetadata {
    pub fn new(id: Ulid, title: String, date: NaiveDate) -> Self {
        Self { id, title, date }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: Ulid,
    pub nickname: String,
    pub email: String,
    pub access_token: AccessToken,
    pub login_method: LoginMethod,
}

impl User {
    pub fn new(
        id: Ulid,
        nickname: String,
        email: String,
        access_token: AccessToken,
        login_method: LoginMethod,
    ) -> Self {
        Self {
            id,
            nickname,
            email,
            access_token,
            login_method,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessToken {
    token: String,
    expire_at: DateTime<Utc>,
}

impl AccessToken {
    pub fn generate_new() -> Self {
        Self {
            token: Alphanumeric.sample_string(&mut rand::thread_rng(), 32),
            expire_at: Utc::now() + chrono::Duration::days(1),
        }
    }

    pub fn new(token: String, expire_at: DateTime<Utc>) -> Self {
        Self { token, expire_at }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expire_at
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn expire_at(&self) -> &DateTime<Utc> {
        &self.expire_at
    }
}

impl Debug for AccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AccessToken{{token: {}***, expire_at: {}}}",
            self.token.chars().take(6).collect::<String>(),
            self.expire_at
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginMethod {
    Github {
        access_token: AccessToken,
        refresh_token: AccessToken,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paper {
    pub id: Ulid,
    pub email: String,
    pub user_id: Ulid,
    pub title: String,
    pub description: String,
    pub speaker: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vote {
    pub paper_id: Ulid,
    pub meet_up_id: Ulid,
    pub user_id: Ulid,
    pub vote: u32,
}

#[derive(Debug)]
pub struct VoteDecider {
    votes: Vec<Vote>,
}

impl VoteDecider {
    pub fn new(votes: Vec<Vote>) -> Self {
        Self { votes }
    }

    pub fn decide(&self) -> Option<Ulid> {
        let votes_per_user = self
            .votes
            .iter()
            .fold(
                HashMap::new(),
                |mut acc: HashMap<Ulid, Vec<&Vote>>, vote| {
                    acc.entry(vote.user_id).or_default().push(vote);
                    acc
                },
            )
            .into_iter()
            .map(|(user_id, mut votes)| {
                votes.sort_by(|a, b| a.vote.cmp(&b.vote));
                (user_id, votes)
            })
            .collect();
        decide_which_paper(votes_per_user)
    }
}

/// Not the most efficient algorithm. But it works and should be enough for now.
fn decide_which_paper(votes_per_user: HashMap<Ulid, Vec<&Vote>>) -> Option<Ulid> {
    let paper_votes = votes_per_user
        .values()
        .filter_map(|votes| votes.first().map(|vote| vote.paper_id))
        .fold(HashMap::new(), |mut acc, paper_id| {
            *acc.entry(paper_id).or_insert(0) += 1;
            acc
        });
    if paper_votes.len() == 1 {
        return paper_votes.keys().next().copied();
    }
    let least_voted_paper = paper_votes
        .iter()
        .min_by_key(|(_, votes)| *votes)
        .map(|(paper_id, _)| paper_id)?;
    let votes_per_user = votes_per_user
        .into_iter()
        .map(|(user_id, mut votes)| {
            if votes
                .first()
                .map(|vote| vote.paper_id == *least_voted_paper)
                .unwrap_or(false)
            {
                let _ = votes.remove(0);
                return (user_id, votes);
            }
            (user_id, votes)
        })
        .collect();
    decide_which_paper(votes_per_user)
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! vote {
        () => {
            Vote {
                paper_id: Ulid::new(),
                meet_up_id: Ulid::new(),
                user_id: Ulid::new(),
                vote: 0,
            }
        };
        ($x:ident: $y:expr) => {
            vote!($x: $y,)
        };
        (user_id: $user_id:expr, $($x:ident: $y:expr),*) => {
            {
                let mut vote = vote!($($x: $y),*);
                vote.user_id = $user_id;
                vote
            }
        };
        (paper_id: $paper_id:expr, $($x:ident: $y:expr),*) => {
            {
                let mut vote = vote!($($x: $y),*);
                vote.paper_id = $paper_id;
                vote
            }
        };
        (vote: $vote_value:expr, $($x:ident: $y:expr),*) => {
            {
                let mut vote = vote!($($x: $y),*);
                vote.vote = $vote_value;
                vote
            }
        };
    }

    #[test]
    fn no_votes() {
        let votes = vec![];
        let vote_decider = VoteDecider::new(votes);
        assert_eq!(vote_decider.decide(), None);
    }

    #[test]
    fn single_votes() {
        let paper_id = Ulid::new();
        let votes = vec![vote!(paper_id: paper_id)];

        let vote_decider = VoteDecider::new(votes);
        assert_eq!(vote_decider.decide(), Some(paper_id));
    }

    #[test]
    fn multiple_single_votes_for_same_paper() {
        let paper_id = Ulid::new();
        let votes = vec![vote!(paper_id: paper_id,), vote!(paper_id: paper_id)];

        let vote_decider = VoteDecider::new(votes);
        assert_eq!(vote_decider.decide(), Some(paper_id));
    }

    /// In case of a draw, any paper can be selected due to HashMap ordering.
    #[test]
    fn multiple_single_votes_for_different_paper_draw() {
        let paper_id_1 = Ulid::new();
        let paper_id_2 = Ulid::new();
        let votes = vec![vote!(paper_id: paper_id_1,), vote!(paper_id: paper_id_2)];

        let vote_decider = VoteDecider::new(votes);
        let winner = vote_decider.decide();
        assert!(winner == Some(paper_id_1) || winner == Some(paper_id_2));
    }

    #[test]
    fn draw_with_more_losers() {
        let paper_id_1 = Ulid::new();
        let paper_id_2 = Ulid::new();
        let paper_id_3 = Ulid::new();
        let votes = vec![
            vote!(paper_id: paper_id_1),
            vote!(paper_id: paper_id_1),
            vote!(paper_id: paper_id_2),
            vote!(paper_id: paper_id_2),
            vote!(paper_id: paper_id_3),
        ];

        let vote_decider = VoteDecider::new(votes);
        let winner = vote_decider.decide();
        assert!(winner == Some(paper_id_1) || winner == Some(paper_id_2));
        assert_ne!(winner, Some(paper_id_3));
    }

    #[test]
    fn multiple_votes() {
        let paper_id_1 = Ulid::new();
        let user_id_1 = Ulid::new();
        let user_id_2 = Ulid::new();
        let user_id_3 = Ulid::new();
        let paper_id_2 = Ulid::new();
        let votes = vec![
            vote!(paper_id: paper_id_1, user_id: user_id_1, vote: 0),
            vote!(paper_id: paper_id_2, user_id: user_id_1, vote: 1),
            vote!(paper_id: paper_id_2, user_id: user_id_2, vote: 0),
            vote!(paper_id: paper_id_1, user_id: user_id_2, vote: 1),
            vote!(paper_id: paper_id_1, user_id: user_id_3, vote: 0),
            vote!(paper_id: paper_id_2, user_id: user_id_3, vote: 1),
        ];

        let vote_decider = VoteDecider::new(votes);
        assert_eq!(vote_decider.decide(), Some(paper_id_1));
    }

    #[test]
    fn most_voted_loses_after_some_itrations() {
        let paper_id_1 = Ulid::new();
        let paper_id_2 = Ulid::new();
        let paper_id_3 = Ulid::new();
        let paper_id_4 = Ulid::new();
        let user_id_1 = Ulid::new();
        let user_id_2 = Ulid::new();
        let user_id_3 = Ulid::new();
        let user_id_4 = Ulid::new();
        let user_id_5 = Ulid::new();

        let votes = vec![
            vote!(paper_id: paper_id_1, user_id: user_id_1, vote: 0),
            vote!(paper_id: paper_id_2, user_id: user_id_1, vote: 1),
            vote!(paper_id: paper_id_1, user_id: user_id_2, vote: 0),
            vote!(paper_id: paper_id_2, user_id: user_id_2, vote: 1),
            vote!(paper_id: paper_id_2, user_id: user_id_3, vote: 0),
            vote!(paper_id: paper_id_2, user_id: user_id_3, vote: 1),
            vote!(paper_id: paper_id_3, user_id: user_id_4, vote: 0),
            vote!(paper_id: paper_id_2, user_id: user_id_4, vote: 1),
            vote!(paper_id: paper_id_4, user_id: user_id_5, vote: 0),
            vote!(paper_id: paper_id_2, user_id: user_id_5, vote: 1),
        ];
        let vote_decider = VoteDecider::new(votes);
        assert_eq!(vote_decider.decide(), Some(paper_id_2));
    }

    /// We don't really expect this to be true. But it is important to support this case.
    #[test]
    fn different_number_of_votes_per_user() {
        let paper_id_1 = Ulid::new();
        let paper_id_2 = Ulid::new();
        let user_id_1 = Ulid::new();
        let user_id_2 = Ulid::new();

        let votes = vec![
            vote!(paper_id: paper_id_1, user_id: user_id_1, vote: 0),
            vote!(paper_id: paper_id_2, user_id: user_id_1, vote: 1),
            vote!(paper_id: paper_id_2, user_id: user_id_2, vote: 0),
        ];

        let vote_decider = VoteDecider::new(votes);
        assert_eq!(vote_decider.decide(), Some(paper_id_1));
    }
}
