use std::sync::Arc;

use axum::{extract::State, Form, response::Html};
use axum::extract::Path;
use axum_htmx::HxRequest;
use minijinja::context;
use ulid::Ulid;

use domain::{get_paper, show_voting, store_votes};

use crate::{app::AppState, controllers::UserPresenter, extractors::LoggedUser};

use super::HtmlError;

pub async fn voting(
    user: LoggedUser,
    HxRequest(is_hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state.get_minijinja_env().get_template("voting")?;

    let (future_meet_up, papers) = show_voting(
        &state.database_gateway,
        &state.database_gateway,
        &state.database_gateway,
        &user.0.id,
    )
        .await?;

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
    _: HxRequest,
    State(state): State<Arc<AppState>>,
    Form(form): Form<Vec<(String, Ulid)>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state.get_minijinja_env().get_template("voting")?;
    store_votes(
        &state.database_gateway,
        &state.database_gateway,
        &user.0.id,
        form.into_iter().map(|(_, paper_id)| {
            paper_id
        }).collect(),
    ).await?;
    let (future_meet_up, papers) = show_voting(
        &state.database_gateway,
        &state.database_gateway,
        &state.database_gateway,
        &user.0.id,
    )
        .await?;
    let context = context! {
        user => UserPresenter::from(user.0),
        client_id => state.github_client_id.clone(),
        future_meet_up => future_meet_up,
        papers => papers,
        errors => Vec::<String>::new(),
    };
    Ok(Html(tmpl.eval_to_state(context)?.render_block("papers")?))
}

pub async fn paper_details(
    Path(paper_id): Path<Ulid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let paper = get_paper(&state.database_gateway, &paper_id).await?;
    let tmpl = state.get_minijinja_env().get_template("components/vote_paper/paper_details")?;
    let context = context! {
        paper => paper,
    };
    Ok(Html(tmpl.render(context)?))
}

pub async fn paper_no_details(
    Path(paper_id): Path<Ulid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let paper = get_paper(&state.database_gateway, &paper_id).await?;
    let tmpl = state.get_minijinja_env().get_template("components/vote_paper/paper")?;
    let context = context! {
        paper => paper,
    };
    Ok(Html(tmpl.render(context)?))
}