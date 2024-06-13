use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use axum::{extract::State, response::Html};
use axum::extract::Path as PathExtractor;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use axum_htmx::HxRequest;
use chrono::NaiveDate;
use minijinja::context;
use minijinja::Environment;
use serde::{Deserialize, Serialize};
use tower_http::services::ServeDir;
use ulid::Ulid;
use url::Url;

use domain::show_home_page;
use gateway::SqliteDatabaseGateway;

pub async fn build_app<T: Clone + Send + Sync + 'static>(
    assets_dir: impl AsRef<Path>,
    database_url: String,
) -> Result<Router<T>> {
    Ok(Router::new()
        .route("/", get(index))
        .route("/pastMeetUp/:id", get(past_meet_up))
        .route("/pastMeetUp/metadata/:id", get(past_meet_up_metadata))
        .with_state(Arc::new(AppState::new(
            SqliteDatabaseGateway::new(database_url).await?,
        )?))
        .fallback_service(ServeDir::new(assets_dir.as_ref())))
}

pub async fn index(
    HxRequest(is_hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state.get_minijinja_env().get_template("home")?;
    let (future_meet_up, past_meet_ups) = show_home_page(&state.database_gateway, &state.database_gateway).await?;

    let context = context! {
        future_meet_up => future_meet_up,
        past_meetups => past_meet_ups,
    };
    match is_hx_request {
        true => Ok(Html(tmpl.eval_to_state(context)?.render_block("content")?)),
        false => Ok(Html(tmpl.render(context)?)),
    }
}

pub async fn past_meet_up(
    PathExtractor(id): PathExtractor<Ulid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state
        .get_minijinja_env()
        .get_template("components/past_meet_up")?;
    let meetup = PastMeetUp {
        id: Ulid::new(),
        title: "Rust Meetup".to_string(),
        date: NaiveDate::from_ymd_opt(2021, 10, 10).unwrap(),
        description: "This is a Rust Meetup".to_string(),
        speaker: "Bruno Clemente".to_string(),
        link: Url::parse("https://www.rust-lang.org").unwrap(),
    };
    let context = context! {
        meetup => meetup,
    };
    Ok(Html(tmpl.render(context)?))
}

pub async fn past_meet_up_metadata(
    PathExtractor(id): PathExtractor<Ulid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state
        .get_minijinja_env()
        .get_template("components/past_meet_up_metadata")?;
    let meetup = PastMeetUpMetadata {
        id: Ulid::new(),
        title: "Rust Meetup".to_string(),
        date: NaiveDate::from_ymd_opt(2021, 10, 10).unwrap(),
    };
    let context = context! {
        meetup => meetup,
    };
    Ok(Html(tmpl.render(context)?))
}

impl<E> From<E> for HtmlError where E: std::fmt::Display {
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
    database_gateway: SqliteDatabaseGateway,
    minijinja_enviroment: Environment<'static>,
}

impl AppState {
    pub fn new(database_gateway: SqliteDatabaseGateway) -> Result<Self> {
        let mut env = Environment::new();
        env.add_template("base", include_str!("templates/base.html"))?;
        env.add_template("home", include_str!("templates/home.html"))?;
        env.add_template(
            "components/past_meet_ups",
            include_str!("templates/components/past_meet_ups.html"),
        )?;
        env.add_template(
            "components/past_meet_up_metadata",
            include_str!("templates/components/past_meet_up_metadata.html"),
        )?;
        env.add_template(
            "components/past_meet_up",
            include_str!("templates/components/past_meet_up.html"),
        )?;
        Ok(Self {
            database_gateway,
            minijinja_enviroment: env,
        })
    }

    pub fn get_minijinja_env(&self) -> &Environment<'static> {
        &self.minijinja_enviroment
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PastMeetUpMetadata {
    id: Ulid,
    title: String,
    date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PastMeetUp {
    id: Ulid,
    title: String,
    #[serde(serialize_with = "md_to_html")]
    description: String,
    speaker: String,
    date: NaiveDate,
    link: Url,
}

fn md_to_html<S>(md: &str, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
{
    let html = markdown::to_html(md);
    serializer.serialize_str(&html)
}
