use std::sync::Arc;

use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Router,
};
use axum_htmx::HxRequest;
use chrono::NaiveDate;
use domain::{show_home_page, FutureMeetUp, FutureMeetUpState};
use future_meet_up::create_future_meet_up;
use minijinja::context;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{app::AppState, extractors::AdminUser};

use super::HtmlError;

pub mod future_meet_up;

pub fn admin_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(admin))
        .route("/createFutureMeetUp", post(create_future_meet_up))
}

pub async fn admin(
    _: AdminUser,
    HxRequest(is_hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state.get_minijinja_env().get_template("admin")?;
    let (future_meet_up, _) =
        show_home_page(&state.database_gateway, &state.database_gateway).await?;

    let context = context! {
        future_meet_up => future_meet_up.map(FutureMeetUpPresenter::from),
    };
    match is_hx_request {
        true => Ok(Html(tmpl.eval_to_state(context)?.render_block("content")?)),
        false => Ok(Html(tmpl.render(context)?)),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FutureMeetUpPresenter {
    id: Ulid,
    title: String,
    state: String,
    description: String,
    speaker: String,
    date: NaiveDate,
    location: String,
}

impl From<FutureMeetUp> for FutureMeetUpPresenter {
    fn from(meetup: FutureMeetUp) -> Self {
        let (state, title, description, speaker) = match meetup.state {
            FutureMeetUpState::Scheduled {
                title,
                description,
                speaker,
            } => ("Scheduled".into(), title, description, speaker),
            FutureMeetUpState::CallForPapers => (
                "CallForPapers".into(),
                String::new(),
                String::new(),
                String::new(),
            ),
            FutureMeetUpState::Voting => {
                ("Voting".into(), String::new(), String::new(), String::new())
            }
        };
        Self {
            id: meetup.id,
            title,
            state,
            description,
            speaker,
            date: meetup.date,
            location: meetup.location,
        }
    }
}
