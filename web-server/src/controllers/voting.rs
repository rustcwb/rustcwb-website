use std::{collections::HashMap, sync::Arc};

use axum::{extract::State, response::Html, Form};
use axum_htmx::HxRequest;
use domain::show_voting;
use minijinja::context;
use ulid::Ulid;

use crate::{app::AppState, controllers::UserPresenter, extractors::LoggedUser};

use super::HtmlError;

pub async fn voting(
    user: LoggedUser,
    HxRequest(is_hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state.get_minijinja_env().get_template("voting")?;

    let (future_meet_up, papers) =
        show_voting(&state.database_gateway, &state.database_gateway).await?;

    let context = context! {
        user => UserPresenter::from(user.0),
        client_id => state.github_client_id.clone(),
        future_meet_up => future_meet_up,
        papers => papers,
        errors => Vec::<String>::new(),
    };
    match is_hx_request {
        true => Ok(Html(tmpl.eval_to_state(context)?.render_block("content")?)),
        false => Ok(Html(tmpl.render(context)?)),
    }
}

pub async fn store_vote(
    user: LoggedUser,
    HxRequest(is_hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
    Form(form): Form<Vec<(String, String)>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state.get_minijinja_env().get_template("voting")?;
    dbg!(form);

    let (future_meet_up, papers) =
        show_voting(&state.database_gateway, &state.database_gateway).await?;

    let context = context! {
        user => UserPresenter::from(user.0),
        client_id => state.github_client_id.clone(),
        future_meet_up => future_meet_up,
        papers => papers,
        errors => Vec::<String>::new(),
    };
    match is_hx_request {
        true => Ok(Html(tmpl.eval_to_state(context)?.render_block("content")?)),
        false => Ok(Html(tmpl.render(context)?)),
    }
}
