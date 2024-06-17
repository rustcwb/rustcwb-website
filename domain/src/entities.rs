use chrono::NaiveDate;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FutureMeetUpState {
    CallForPapers,
    Voting,
    Scheduled {
        title: String,
        description: String,
        speaker: String,
    },
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
