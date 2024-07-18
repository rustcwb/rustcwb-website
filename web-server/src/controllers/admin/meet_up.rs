use anyhow::anyhow;
use axum::{extract::State, response::Html, Form};
use chrono::{DateTime, NaiveDateTime, Utc};
use minijinja::context;
use serde::{Deserialize, Deserializer};
use std::sync::Arc;
use url::Url;

use domain::{
    create_new_meet_up, move_future_meet_up_to_done, move_future_meet_up_to_scheduled,
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
    let location = match params.location_type.as_str() {
        "OnSite" => domain::Location::OnSite(params.location_address),
        "Online" => domain::Location::Online {
            video_conference_link: params.location_video_conference_link.parse()?,
            calendar_link: params.location_calendar_link.parse()?,
        },
        other => return Err(anyhow!("Invalid location type {other}").into()),
    };
    let meet_up = create_new_meet_up(&state.database_gateway, location, params.date).await?;

    let context = context! {
        future_meet_up => MeetUpPresenter::from(meet_up),
        client_id => state.github_client_id.clone(),
    };
    Ok(Html(tmpl.render(context)?))
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateFutureMeetUpParam {
    location_type: String,
    location_address: String,
    location_video_conference_link: String,
    location_calendar_link: String,
    #[serde(deserialize_with = "from_datetime_local_form")]
    date: DateTime<Utc>,
}

fn from_datetime_local_form<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M")
        .map_err(serde::de::Error::custom)
        .and_then(|dt| {
            dt.and_local_timezone(chrono_tz::Brazil::West)
                .single()
                .map(|dt| dt.to_utc())
                .ok_or(serde::de::Error::custom("Invalid date"))
        })
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
    move_future_meet_up_to_done(&state.database_gateway, params.link).await?;

    let context = context! {
        client_id => state.github_client_id.clone(),
    };
    Ok(Html(tmpl.render(context)?))
}

#[derive(Debug, Clone, Deserialize)]
pub struct FinishFutureMeetUpParam {
    link: Url,
}
