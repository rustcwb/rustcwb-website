use std::sync::Arc;

use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Router,
};
use axum_htmx::HxRequest;
use minijinja::context;

use domain::show_admin_page;
use meet_up::{create_meet_up, finish, go_for_voting, schedule};

use crate::extractors::MaybeUser;
use crate::{app::AppState, controllers::MeetUpPresenter, extractors::AdminUser};

use super::{HtmlError, UserPresenter};

pub mod meet_up;

pub fn admin_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(admin))
        .route("/createMeetUp", post(create_meet_up))
        .route("/voting", post(go_for_voting))
        .route("/schedule", post(schedule))
        .route("/finish", post(finish))
}

pub async fn admin(
    _: AdminUser,
    maybe_user: MaybeUser,
    HxRequest(is_hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state.get_minijinja_env().get_template("admin")?;
    let meet_up_response = show_admin_page(
        &state.database_gateway,
        &state.database_gateway,
        &state.database_gateway,
    )
    .await?;

    let context = context! {
        n_papers => meet_up_response.n_papers(),
        n_attendees => meet_up_response.n_attendees(),
        future_meet_up => meet_up_response.into_meet_up().map(MeetUpPresenter::from),
        user => maybe_user.0.map(UserPresenter::from),
        client_id => state.github_client_id.clone(),
    };
    match is_hx_request {
        true => Ok(Html(tmpl.eval_to_state(context)?.render_block("content")?)),
        false => Ok(Html(tmpl.render(context)?)),
    }
}
