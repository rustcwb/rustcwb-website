use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastMeetUp {
    id: Ulid,
    title: String,
    description: String,
    speaker: String,
    date: NaiveDate,
    link: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastMeetUpMetadata {
    id: Ulid,
    title: String,
    date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FutureMeetUp {
    id: Ulid,
    state: FutureMeetUpState,
    date: NaiveDate,
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
