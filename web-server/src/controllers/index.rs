use std::sync::Arc;

use axum::{extract::State, response::Html};
use axum_htmx::HxRequest;
use minijinja::context;

use domain::show_home_page;

use crate::{
    app::AppState,
    controllers::{MeetUpPresenter, UserPresenter},
    extractors::MaybeUser,
};

use super::{HtmlError, MeetUpMetadataPresenter};

pub async fn index(
    maybe_user: MaybeUser,
    HxRequest(is_hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state.get_minijinja_env().get_template("home")?;
    let (future_meet_up, past_meet_ups, is_registered_user) = show_home_page(
        &state.database_gateway,
        &state.database_gateway,
        maybe_user.0.as_ref().map(|user| &user.id),
    )
    .await?;

    let context = context! {
        user => maybe_user.0.map(UserPresenter::from),
        registered_user => is_registered_user,
        client_id => state.github_client_id.clone(),
        future_meet_up => future_meet_up.map(MeetUpPresenter::from),
        past_meetups => past_meet_ups.into_iter().map(MeetUpMetadataPresenter::from).collect::<Vec<MeetUpMetadataPresenter>>(),
    };
    match is_hx_request {
        true => Ok(Html(tmpl.eval_to_state(context)?.render_block("content")?)),
        false => Ok(Html(tmpl.render(context)?)),
    }
}
