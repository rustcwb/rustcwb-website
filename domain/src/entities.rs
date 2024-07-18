use std::{collections::HashMap, fmt::Debug};

use chrono::{DateTime, NaiveDate, Utc};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

use shared::utc_now;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MeetUpMetadata {
    pub id: Ulid,
    pub title: String,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeetUp {
    pub id: Ulid,
    pub state: MeetUpState,
    pub location: Location,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Location {
    Online {
        video_conference_link: Url,
        calendar_link: Url,
    },
    OnSite(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeetUpState {
    CallForPapers,
    Voting,
    Scheduled(Paper),
    Done { paper: Paper, link: Url },
}

impl std::fmt::Display for MeetUpState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeetUpState::CallForPapers => write!(f, "CallForPapers"),
            MeetUpState::Voting => write!(f, "Voting"),
            MeetUpState::Scheduled { .. } => write!(f, "Scheduled"),
            MeetUpState::Done { .. } => write!(f, "Done"),
        }
    }
}

impl MeetUp {
    pub fn new(id: Ulid, state: MeetUpState, location: Location, date: NaiveDate) -> Self {
        Self {
            id,
            state,
            location,
            date,
        }
    }
}

impl MeetUpMetadata {
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
            expire_at: utc_now() + chrono::Duration::days(1),
        }
    }

    pub fn new(token: String, expire_at: DateTime<Utc>) -> Self {
        Self { token, expire_at }
    }

    pub fn is_expired(&self) -> bool {
        utc_now() > self.expire_at
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vote {
    pub paper_id: Ulid,
    pub meet_up_id: Ulid,
    pub user_id: Ulid,
    pub vote: f64,
}

#[derive(Debug)]
pub struct VoteDecider {
    votes: Vec<Vote>,
}

impl VoteDecider {
    pub fn new(votes: Vec<Vote>) -> Self {
        Self { votes }
    }

    /// We use harmonic positional voting https://en.wikipedia.org/wiki/Positional_voting
    pub fn decide(&self) -> Option<Ulid> {
        let votes_per_paper_id: HashMap<Ulid, f64> =
            self.votes
                .iter()
                .fold(HashMap::new(), |mut acc: HashMap<Ulid, f64>, vote| {
                    *acc.entry(vote.paper_id).or_default() += vote.vote;
                    acc
                });
        let (paper_id, _) = votes_per_paper_id
            .into_iter()
            .max_by(|(_, vote_a), (_, vote_b)| {
                vote_a
                    .partial_cmp(vote_b)
                    .unwrap_or(std::cmp::Ordering::Less)
            })?;
        Some(paper_id)
    }
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
                vote: 1.0,
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
            vote!(paper_id: paper_id_1, user_id: user_id_1, vote: 1.0),
            vote!(paper_id: paper_id_2, user_id: user_id_1, vote: 0.5),
            vote!(paper_id: paper_id_2, user_id: user_id_2, vote: 1.0),
            vote!(paper_id: paper_id_1, user_id: user_id_2, vote: 0.5),
            vote!(paper_id: paper_id_1, user_id: user_id_3, vote: 1.0),
            vote!(paper_id: paper_id_2, user_id: user_id_3, vote: 0.5),
        ];

        let vote_decider = VoteDecider::new(votes);
        assert_eq!(vote_decider.decide(), Some(paper_id_1));
    }

    #[test]
    fn multiple_votes_and_papers() {
        let paper_id_1 = dbg!(Ulid::new());
        let paper_id_2 = dbg!(Ulid::new());
        let paper_id_3 = dbg!(Ulid::new());
        let user_id_1 = Ulid::new();
        let user_id_2 = Ulid::new();
        let user_id_3 = Ulid::new();
        let user_id_4 = Ulid::new();
        let user_id_5 = Ulid::new();
        let user_id_6 = Ulid::new();
        let user_id_7 = Ulid::new();

        let votes = vec![
            vote!(paper_id: paper_id_1, user_id: user_id_1, vote: 1.0),
            vote!(paper_id: paper_id_2, user_id: user_id_1, vote: 0.5),
            vote!(paper_id: paper_id_3, user_id: user_id_1, vote: 0.25),
            vote!(paper_id: paper_id_2, user_id: user_id_2, vote: 1.0),
            vote!(paper_id: paper_id_1, user_id: user_id_2, vote: 0.5),
            vote!(paper_id: paper_id_3, user_id: user_id_2, vote: 0.25),
            vote!(paper_id: paper_id_1, user_id: user_id_3, vote: 1.0),
            vote!(paper_id: paper_id_2, user_id: user_id_3, vote: 0.5),
            vote!(paper_id: paper_id_3, user_id: user_id_3, vote: 0.25),
            vote!(paper_id: paper_id_3, user_id: user_id_4, vote: 1.0),
            vote!(paper_id: paper_id_2, user_id: user_id_4, vote: 0.5),
            vote!(paper_id: paper_id_1, user_id: user_id_4, vote: 0.25),
            vote!(paper_id: paper_id_3, user_id: user_id_5, vote: 1.0),
            vote!(paper_id: paper_id_2, user_id: user_id_5, vote: 0.5),
            vote!(paper_id: paper_id_3, user_id: user_id_5, vote: 0.25),
            vote!(paper_id: paper_id_1, user_id: user_id_6, vote: 1.0),
            vote!(paper_id: paper_id_3, user_id: user_id_6, vote: 0.5),
            vote!(paper_id: paper_id_2, user_id: user_id_6, vote: 0.25),
            vote!(paper_id: paper_id_3, user_id: user_id_7, vote: 1.0),
            vote!(paper_id: paper_id_2, user_id: user_id_7, vote: 0.5),
            vote!(paper_id: paper_id_1, user_id: user_id_7, vote: 0.25),
        ];
        let vote_decider = VoteDecider::new(votes);
        assert_eq!(vote_decider.decide(), Some(paper_id_3));
    }

    /// We don't really expect this to be true. But it is important to support this case.
    #[test]
    fn different_number_of_votes_per_user() {
        let paper_id_1 = Ulid::new();
        let paper_id_2 = Ulid::new();
        let user_id_1 = Ulid::new();
        let user_id_2 = Ulid::new();
        let user_id_3 = Ulid::new();

        let votes = vec![
            vote!(paper_id: paper_id_1, user_id: user_id_1, vote: 1.0),
            vote!(paper_id: paper_id_2, user_id: user_id_1, vote: 0.5),
            vote!(paper_id: paper_id_2, user_id: user_id_2, vote: 1.0),
            vote!(paper_id: paper_id_1, user_id: user_id_3, vote: 1.0),
        ];

        let vote_decider = VoteDecider::new(votes);
        assert_eq!(vote_decider.decide(), Some(paper_id_1));
    }
}
