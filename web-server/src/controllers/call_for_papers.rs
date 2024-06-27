use std::sync::Arc;

use anyhow::anyhow;
use axum::{extract::State, response::Html, Form};
use axum_htmx::HxRequest;
use domain::{show_call_for_papers, submit_paper, Paper, SubmitPaperError};
use minijinja::context;
use serde::Deserialize;
use ulid::Ulid;

use crate::{
    app::AppState,
    controllers::{MeetUpPresenter, UserPresenter},
    extractors::LoggedUser,
};

use super::HtmlError;

pub async fn call_for_papers(
    user: LoggedUser,
    HxRequest(is_hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    call_for_papers_with_errors(&[], user, is_hx_request, &state).await
}

pub async fn save_call_for_papers(
    user: LoggedUser,
    HxRequest(is_hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
    Form(params): Form<PaperParams>,
) -> Result<Html<String>, HtmlError> {
    let mut errors = Vec::new();
    if params.title.is_empty() {
        errors.push("Title is required");
    }
    if params.email.is_empty() {
        errors.push("Email is required");
    }
    if params.description.is_empty() {
        errors.push("Description is required");
    }
    if params.speaker.is_empty() {
        errors.push("Speaker is required");
    }
    if !errors.is_empty() {
        return call_for_papers_with_errors(&errors, user, is_hx_request, &state).await;
    }
    match submit_paper(
        &state.database_gateway,
        &state.database_gateway,
        Paper {
            id: Ulid::new(),
            title: params.title,
            email: params.email,
            description: params.description,
            speaker: params.speaker,
            user_id: user.0.id,
        },
    )
    .await
    {
        Ok(_) => {
            let tmpl = state.get_minijinja_env().get_template("success")?;
            let context = context! { message => "Paper submetido com sucesso"};
            match is_hx_request {
                true => Ok(Html(tmpl.eval_to_state(context)?.render_block("content")?)),
                false => Ok(Html(tmpl.render(context)?)),
            }
        }
        Err(SubmitPaperError::InvalidMeetUpState(_))
        | Err(SubmitPaperError::NoFutureMeetUpFound) => {
            call_for_papers_with_errors(
                &["Meet up is not accepting papers"],
                user,
                is_hx_request,
                &state,
            )
            .await
        }
        Err(SubmitPaperError::MoreThanLimitPapersPerUserPerMeetUp(_)) => {
            call_for_papers_with_errors(
                &["You have already submitted the limit of papers for this meet up"],
                user,
                is_hx_request,
                &state,
            )
            .await
        }
        Err(SubmitPaperError::Unknown(err)) => Err(HtmlError::from(anyhow!("{err}"))),
    }
}

async fn call_for_papers_with_errors(
    errors: &[&str],
    user: LoggedUser,
    is_hx_request: bool,
    state: &AppState,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state.get_minijinja_env().get_template("call_for_papers")?;
    let (future_meet_up, papers, is_papers_limit) =
        show_call_for_papers(&state.database_gateway, &state.database_gateway, &user.0).await?;

    let context = context! {
        user => UserPresenter::from(user.0),
        client_id => state.github_client_id.clone(),
        future_meet_up => MeetUpPresenter::from(future_meet_up),
        papers => papers,
        is_papers_limit => is_papers_limit,
        errors => errors,
    };
    match is_hx_request {
        true => Ok(Html(tmpl.eval_to_state(context)?.render_block("content")?)),
        false => Ok(Html(tmpl.render(context)?)),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaperParams {
    pub title: String,
    pub email: String,
    pub description: String,
    pub speaker: String,
}
