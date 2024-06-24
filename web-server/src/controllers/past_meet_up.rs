use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::Html,
};
use chrono::NaiveDate;
use domain::{get_past_meet_up, get_past_meet_up_metadata, PastMeetUp};
use minijinja::context;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

use crate::app::AppState;

use super::HtmlError;

pub async fn past_meet_up(
    Path(id): Path<Ulid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state
        .get_minijinja_env()
        .get_template("components/past_meet_ups/past_meet_up")?;
    let meetup = get_past_meet_up(&state.database_gateway, id).await?;
    let context = context! {
        meetup => PastMeetUpPresenter::from(meetup)
    };
    Ok(Html(tmpl.render(context)?))
}

pub async fn past_meet_up_metadata(
    Path(id): Path<Ulid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state
        .get_minijinja_env()
        .get_template("components/past_meet_ups/past_meet_up_metadata")?;
    let meetup = get_past_meet_up_metadata(&state.database_gateway, id).await?;
    let context = context! {
        meetup => meetup,
    };
    Ok(Html(tmpl.render(context)?))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PastMeetUpPresenter {
    id: Ulid,
    title: String,
    description: String,
    speaker: String,
    date: NaiveDate,
    link: Url,
}

impl From<PastMeetUp> for PastMeetUpPresenter {
    fn from(meetup: PastMeetUp) -> Self {
        Self {
            id: meetup.id,
            title: meetup.paper.title,
            description: markdown::to_html(&meetup.paper.description),
            speaker: meetup.paper.speaker,
            date: meetup.date,
            link: meetup.link,
        }
    }
}
