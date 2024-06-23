use std::fmt::Debug;

use chrono::{DateTime, NaiveDate, Utc};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastMeetUp {
    pub id: Ulid,
    pub title: String,
    pub description: String,
    pub speaker: String,
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
    Scheduled {
        title: String,
        description: String,
        speaker: String,
    },
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
    pub fn new(
        id: Ulid,
        title: String,
        description: String,
        speaker: String,
        date: NaiveDate,
        link: Url,
    ) -> Self {
        Self {
            id,
            title,
            description,
            speaker,
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
