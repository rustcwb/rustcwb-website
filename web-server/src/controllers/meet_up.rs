use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::Html,
};
use minijinja::context;
use ulid::Ulid;

use domain::{get_meet_up, get_meet_up_metadata};

use crate::{app::AppState, controllers::MeetUpPresenter};

use super::{HtmlError, MeetUpMetadataPresenter};

pub async fn meet_up(
    Path(id): Path<Ulid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state
        .get_minijinja_env()
        .get_template("components/past_meet_ups/past_meet_up")?;
    let meetup = get_meet_up(&state.database_gateway, id).await?;
    let context = context! {
        meetup => MeetUpPresenter::from(meetup)
    };
    Ok(Html(tmpl.render(context)?))
}

pub async fn meet_up_metadata(
    Path(id): Path<Ulid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state
        .get_minijinja_env()
        .get_template("components/past_meet_ups/past_meet_up_metadata")?;
    let meetup = get_meet_up_metadata(&state.database_gateway, id).await?;
    let context = context! {
        meetup => MeetUpMetadataPresenter::from(meetup),
    };
    Ok(Html(tmpl.render(context)?))
}
