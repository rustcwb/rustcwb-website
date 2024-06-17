use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use axum::extract::{FromRequestParts, Path as PathExtractor, Query};
use axum::http::header::{AUTHORIZATION, WWW_AUTHENTICATE};
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderName, StatusCode};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{async_trait, Form, Router};
use axum::{extract::State, response::Html};
use axum_extra::extract::cookie::{Cookie, Expiration};
use axum_extra::extract::CookieJar;
use axum_htmx::HxRequest;
use base64::prelude::*;
use chrono::{Duration, NaiveDate, Utc};
use domain::{login_with_access_token, login_with_github_code};
use gateway::github::GithubRestGateway;
use minijinja::context;
use minijinja::Environment;
use serde::{Deserialize, Serialize};
use tower_http::services::ServeDir;
use ulid::Ulid;
use url::Url;

use domain::{
    create_new_future_meet_up, get_past_meet_up, get_past_meet_up_metadata, show_home_page,
    FutureMeetUp, FutureMeetUpState, PastMeetUp, User,
};
use gateway::SqliteDatabaseGateway;

pub async fn build_app<T: Clone + Send + Sync + 'static>(
    assets_dir: impl AsRef<Path>,
    database_url: String,
    admin_details: (String, String),
    (client_id, client_secret): (String, String),
) -> Result<Router<T>> {
    Ok(Router::new()
        .route("/", get(index))
        .route("/github/authorize", get(github_login))
        .route("/admin", get(admin))
        .route("/admin/createFutureMeetUp", post(create_future_meet_up))
        .route("/pastMeetUp/:id", get(past_meet_up))
        .route("/pastMeetUp/metadata/:id", get(past_meet_up_metadata))
        .with_state(Arc::new(AppState::new(
            SqliteDatabaseGateway::new(database_url).await?,
            GithubRestGateway::new(client_id.clone(), client_secret),
            client_id,
            admin_details,
        )?))
        .fallback_service(ServeDir::new(assets_dir.as_ref())))
}

pub async fn index(
    maybe_user: MaybeUser,
    HxRequest(is_hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state.get_minijinja_env().get_template("home")?;
    let (future_meet_up, past_meet_ups) =
        show_home_page(&state.database_gateway, &state.database_gateway).await?;

    let context = context! {
        user => maybe_user.0.map(|u| UserPresenter::from(u)),
        client_id => state.github_client_id.clone(),
        future_meet_up => future_meet_up,
        past_meetups => past_meet_ups,
    };
    match is_hx_request {
        true => Ok(Html(tmpl.eval_to_state(context)?.render_block("content")?)),
        false => Ok(Html(tmpl.render(context)?)),
    }
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
    Ok(dbg!((cookie_jar.add(cookie), Redirect::to("/"))))
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
        future_meet_up => future_meet_up.map(|fut| FutureMeetUpPresenter::from(fut)),
    };
    match is_hx_request {
        true => Ok(Html(tmpl.eval_to_state(context)?.render_block("content")?)),
        false => Ok(Html(tmpl.render(context)?)),
    }
}

pub async fn create_future_meet_up(
    _: AdminUser,
    State(state): State<Arc<AppState>>,
    Form(params): Form<CreateFutureMeetUpParam>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state
        .get_minijinja_env()
        .get_template("components/admin/future_meet_up/future_meet_up")?;
    let meet_up =
        create_new_future_meet_up(&state.database_gateway, params.location, params.date).await?;

    let context = context! {
        future_meet_up => FutureMeetUpPresenter::from(meet_up),
    };
    Ok(Html(tmpl.render(context)?))
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateFutureMeetUpParam {
    location: String,
    date: NaiveDate,
}

pub async fn past_meet_up(
    PathExtractor(id): PathExtractor<Ulid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state
        .get_minijinja_env()
        .get_template("components/past_meet_ups/past_meet_up")?;
    let meetup = get_past_meet_up(&state.database_gateway, id).await?;
    let context = context! {
        meetup => PastMeetUpPresenter::from(meetup)
    };
    Ok(Html(tmpl.render(context)?))
}

pub async fn past_meet_up_metadata(
    PathExtractor(id): PathExtractor<Ulid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state
        .get_minijinja_env()
        .get_template("components/past_meet_ups/past_meet_up_metadata")?;
    let meetup = get_past_meet_up_metadata(&state.database_gateway, id).await?;
    let context = context! {
        meetup => meetup,
    };
    Ok(Html(tmpl.render(context)?))
}

impl<E> From<E> for HtmlError
where
    E: std::fmt::Display,
{
    fn from(err: E) -> Self {
        tracing::error!("Unexpected error: {}", err);
        HtmlError
    }
}

pub struct HtmlError;

impl IntoResponse for HtmlError {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        let html = r#"
            <!doctype html>
            <html lang="pt">
                <head>
                    <title>RustCWB - Panic</title>
                    <meta charset="UTF-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                    <link rel="stylesheet" type="text/css" href="/assets/css/styles.css" />
                    <link rel="preconnect" href="https://fonts.googleapis.com" />
                    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
                    <link
                        rel="icon"
                        sizes="16x16"
                        type="image/png"
                        href="/assets/favicon-16x16.png"
                    />
                    <link
                        rel="icon"
                        sizes="32x32"
                        type="image/png"
                        href="/assets/favicon-32x32.png"
                    />
                    <link rel="icon" type="image/svg+xml" href="/assets/favicon.svg" />
                    <link
                        href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:ital,wght@0,100..800;1,100..800&display=swap"
                        rel="stylesheet"
                    />
                    <script src="https://unpkg.com/htmx.org@1.9.12"></script>
                </head>
                <body>
                    <div class="font-jetBrains container mx-auto p-4">
                        <h1 class="text-4xl font-bold">Dont PANIC!!</h1>
                        <p class="text-xl">An error occurred while processing your request.</p>
                    </div>
                </body>
            </html>
        "#;
        Html(html).into_response()
    }
}

pub struct AppState {
    admin_details: (String, String),
    database_gateway: SqliteDatabaseGateway,
    github_gateway: GithubRestGateway,
    github_client_id: String,
    minijinja_enviroment: Environment<'static>,
}

impl AppState {
    pub fn new(
        database_gateway: SqliteDatabaseGateway,
        github_gateway: GithubRestGateway,
        github_client_id: String,
        admin_details: (String, String),
    ) -> Result<Self> {
        let mut env = Environment::new();
        env.add_template("base", include_str!("templates/base.html"))?;
        env.add_template("home", include_str!("templates/home.html"))?;
        env.add_template("admin", include_str!("templates/admin.html"))?;
        env.add_template(
            "components/past_meet_ups/past_meet_ups",
            include_str!("templates/components/past_meet_ups/past_meet_ups.html"),
        )?;
        env.add_template(
            "components/past_meet_ups/past_meet_up_metadata",
            include_str!("templates/components/past_meet_ups/past_meet_up_metadata.html"),
        )?;
        env.add_template(
            "components/past_meet_ups/past_meet_up",
            include_str!("templates/components/past_meet_ups/past_meet_up.html"),
        )?;
        env.add_template(
            "components/admin/future_meet_up/future_meet_up",
            include_str!("templates/components/admin/future_meet_up/future_meet_up.html"),
        )?;
        env.add_template(
            "components/admin/future_meet_up/no_future_meet_up",
            include_str!("templates/components/admin/future_meet_up/no_future_meet_up.html"),
        )?;
        env.add_template(
            "components/admin/future_meet_up/call_for_papers",
            include_str!("templates/components/admin/future_meet_up/call_for_papers.html"),
        )?;

        Ok(Self {
            admin_details,
            github_gateway,
            github_client_id,
            database_gateway,
            minijinja_enviroment: env,
        })
    }

    pub fn get_minijinja_env(&self) -> &Environment<'static> {
        &self.minijinja_enviroment
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PastMeetUpPresenter {
    id: Ulid,
    title: String,
    description: String,
    speaker: String,
    date: NaiveDate,
    link: Url,
}

impl From<PastMeetUp> for PastMeetUpPresenter {
    fn from(meetup: PastMeetUp) -> Self {
        Self {
            id: meetup.id,
            title: meetup.title,
            description: markdown::to_html(&meetup.description),
            speaker: meetup.speaker,
            date: meetup.date,
            link: meetup.link,
        }
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

pub struct AdminUser();

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AdminUser {
    type Rejection = (StatusCode, HeaderMap);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|header| {
                let header = header.to_str().ok()?;
                let encoded = header.strip_prefix("Basic ")?;
                let decoded = BASE64_STANDARD.decode(encoded).ok()?;
                let decoded = String::from_utf8_lossy(&decoded);
                let Some((user, pass)) = decoded.split_once(':') else {
                    return None;
                };
                if user == state.admin_details.0 && pass == state.admin_details.1 {
                    Some(AdminUser)
                } else {
                    None
                }
            })
            .ok_or((
                StatusCode::UNAUTHORIZED,
                headers_map(&[(WWW_AUTHENTICATE, "Basic realm=\"admin\", charset=\"UTF-8\"")])
                    .map_err(|err| {
                        tracing::error!("Error creating headers map: {err}");
                        (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new())
                    })?,
            ))?;
        Ok(AdminUser())
    }
}

#[derive(Debug)]
pub struct MaybeUser(Option<User>);

#[async_trait]
impl FromRequestParts<Arc<AppState>> for MaybeUser {
    type Rejection = (StatusCode, HeaderMap);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let cookie_jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new()))?;
        match cookie_jar.get("access_token") {
            Some(cookie) => {
                let access_token = cookie.value();
                let user = dbg!(
                    login_with_access_token(
                        &state.database_gateway,
                        &state.github_gateway,
                        access_token,
                    )
                    .await
                )
                .ok();
                Ok(MaybeUser(user))
            }
            None => Ok(MaybeUser(None)),
        }
    }
}

fn headers_map(headers: &[(HeaderName, &str)]) -> Result<HeaderMap> {
    let mut header_map = HeaderMap::new();
    for (header_name, value) in headers {
        header_map.insert(header_name.clone(), value.parse()?);
    }
    Ok(header_map)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserPresenter {
    nickname: String,
}

impl From<User> for UserPresenter {
    fn from(user: User) -> Self {
        Self {
            nickname: user.nickname.chars().take(10).collect::<String>(),
        }
    }
}
