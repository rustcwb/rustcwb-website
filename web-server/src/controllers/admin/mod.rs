use std::sync::Arc;

use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Router,
};
use axum_htmx::HxRequest;
use domain::show_admin_page;
use future_meet_up::{create_future_meet_up, finish, go_for_voting, schedule};
use minijinja::context;

use crate::{app::AppState, controllers::FutureMeetUpPresenter, extractors::AdminUser};

use super::HtmlError;

pub mod future_meet_up;

pub fn admin_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(admin))
        .route("/createFutureMeetUp", post(create_future_meet_up))
        .route("/moveFutureMeetUpIntoVoting", post(go_for_voting))
        .route("/schedule", post(schedule))
        .route("/finish", post(finish))
}

pub async fn admin(
    _: AdminUser,
    HxRequest(is_hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state.get_minijinja_env().get_template("admin")?;
    let (future_meet_up, n_papers) =
        show_admin_page(&state.database_gateway, &state.database_gateway).await?;

    let context = context! {
        future_meet_up => future_meet_up.map(FutureMeetUpPresenter::from),
        n_papers => n_papers,
        client_id => state.github_client_id.clone(),
    };
    match is_hx_request {
        true => Ok(Html(tmpl.eval_to_state(context)?.render_block("content")?)),
        false => Ok(Html(tmpl.render(context)?)),
    }
}
