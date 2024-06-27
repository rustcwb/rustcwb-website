use std::sync::Arc;

use axum::{extract::State, response::Html};
use axum_htmx::HxRequest;
use domain::show_home_page;
use minijinja::context;

use crate::{
    app::AppState,
    controllers::{MeetUpPresenter, UserPresenter},
    extractors::MaybeUser,
};

use super::HtmlError;

pub async fn index(
    maybe_user: MaybeUser,
    HxRequest(is_hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state.get_minijinja_env().get_template("home")?;
    let (future_meet_up, past_meet_ups) = show_home_page(&state.database_gateway).await?;

    let context = context! {
        user => maybe_user.0.map(UserPresenter::from),
        client_id => state.github_client_id.clone(),
        future_meet_up => future_meet_up.map(MeetUpPresenter::from),
        past_meetups => past_meet_ups,
    };
    match is_hx_request {
        true => Ok(Html(tmpl.eval_to_state(context)?.render_block("content")?)),
        false => Ok(Html(tmpl.render(context)?)),
    }
}
