use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use axum::{
    extract::{Query, State},
    response::{Html, Redirect},
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum_htmx::HxRequest;
use domain::login_with_github_code;
use minijinja::context;

use crate::{app::AppState, controllers::UserPresenter, extractors::MaybeUser};

use super::HtmlError;

pub async fn user(
    maybe_user: MaybeUser,
    HxRequest(is_hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state.get_minijinja_env().get_template("user")?;

    let context = context! {
        user => maybe_user.0.map(UserPresenter::from),
        client_id => state.github_client_id.clone(),
    };
    match is_hx_request {
        true => Ok(Html(tmpl.eval_to_state(context)?.render_block("content")?)),
        false => Ok(Html(tmpl.render(context)?)),
    }
}

pub async fn logout(
    cookie_jar: CookieJar,
    _: State<Arc<AppState>>,
) -> Result<(CookieJar, Redirect), HtmlError> {
    Ok((cookie_jar.remove("access_token"), Redirect::to("/")))
}

pub async fn github_login(
    Query(query): Query<HashMap<String, String>>,
    cookie_jar: CookieJar,
    State(state): State<Arc<AppState>>,
) -> Result<(CookieJar, Redirect), HtmlError> {
    let code = query
        .get("code")
        .ok_or_else(|| anyhow!("No code in query"))?
        .to_string();
    let user = login_with_github_code(&state.database_gateway, &state.github_gateway, code).await?;
    // TODO: Add an expiration time
    let mut cookie = Cookie::new("access_token", user.access_token.token().to_string());
    cookie.set_path("/");
    Ok((cookie_jar.add(cookie), Redirect::to("/")))
}
