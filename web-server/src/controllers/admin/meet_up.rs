use std::sync::Arc;

use axum::{extract::State, Form, response::Html};
use chrono::NaiveDate;
use minijinja::context;
use serde::Deserialize;
use url::Url;

use domain::{
    create_new_meet_up, move_future_meet_up_to_past_meet_up, move_future_meet_up_to_scheduled,
    move_future_meet_up_to_voting,
};

use crate::{
    app::AppState,
    controllers::{admin::MeetUpPresenter, HtmlError},
    extractors::AdminUser,
};

pub async fn create_meet_up(
    _: AdminUser,
    State(state): State<Arc<AppState>>,
    Form(params): Form<CreateFutureMeetUpParam>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state
        .get_minijinja_env()
        .get_template("components/admin/future_meet_up/future_meet_up")?;
    let meet_up = create_new_meet_up(&state.database_gateway, params.location, params.date).await?;

    let context = context! {
        future_meet_up => MeetUpPresenter::from(meet_up),
        client_id => state.github_client_id.clone(),
    };
    Ok(Html(tmpl.render(context)?))
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateFutureMeetUpParam {
    location: String,
    date: NaiveDate,
}

pub async fn go_for_voting(
    _: AdminUser,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state
        .get_minijinja_env()
        .get_template("components/admin/future_meet_up/future_meet_up")?;
    let meet_up = move_future_meet_up_to_voting(&state.database_gateway).await?;

    let context = context! {
        future_meet_up => MeetUpPresenter::from(meet_up),
        client_id => state.github_client_id.clone(),
    };
    Ok(Html(tmpl.render(context)?))
}

pub async fn schedule(
    _: AdminUser,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state
        .get_minijinja_env()
        .get_template("components/admin/future_meet_up/future_meet_up")?;
    let meet_up =
        move_future_meet_up_to_scheduled(&state.database_gateway, &state.database_gateway).await?;

    let context = context! {
        future_meet_up => MeetUpPresenter::from(meet_up),
        client_id => state.github_client_id.clone(),
    };
    Ok(Html(tmpl.render(context)?))
}

pub async fn finish(
    _: AdminUser,
    State(state): State<Arc<AppState>>,
    Form(params): Form<FinishFutureMeetUpParam>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state
        .get_minijinja_env()
        .get_template("components/admin/future_meet_up/future_meet_up")?;
    move_future_meet_up_to_past_meet_up(&state.database_gateway, params.link).await?;

    let context = context! {
        client_id => state.github_client_id.clone(),
    };
    Ok(Html(tmpl.render(context)?))
}

#[derive(Debug, Clone, Deserialize)]
pub struct FinishFutureMeetUpParam {
    link: Url,
}
