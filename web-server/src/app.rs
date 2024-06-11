use anyhow::Result;
use axum::extract::Path as PathExtractor;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use axum::{extract::State, response::Html};
use axum_htmx::HxRequest;
use chrono::NaiveDate;
use minijinja::context;
use minijinja::Environment;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tower_http::services::ServeDir;
use ulid::Ulid;
use url::Url;

pub fn build_app<T: Clone + Send + Sync + 'static>(
    assets_dir: impl AsRef<Path>,
) -> Result<Router<T>> {
    Ok(Router::new()
        .route("/", get(index))
        .route("/pastMeetUp/:id", get(past_meet_up))
        .route("/pastMeetUp/metadata/:id", get(past_meet_up_metadata))
        .with_state(Arc::new(AppState::new()?))
        .fallback_service(ServeDir::new(assets_dir.as_ref())))
}

pub async fn index(
    HxRequest(is_hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state.get_minijinja_env().get_template("home")?;
    let vec: Vec<PastMeetUpMetadata> = vec![
        PastMeetUpMetadata {
            id: Ulid::new(),
            title: "Rust Meetup".to_string(),
            date: NaiveDate::from_ymd_opt(2021, 10, 10).unwrap(),
        },
        PastMeetUpMetadata {
            id: Ulid::new(),
            title: "Rust Meetup 2".to_string(),
            date: NaiveDate::from_ymd_opt(2021, 09, 10).unwrap(),
        },
    ];
    let context = context! {
        past_meetups => vec,
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

impl From<minijinja::Error> for HtmlError {
    fn from(e: minijinja::Error) -> Self {
        tracing::error!("Error rendering template: {e}");
        HtmlError
    }
}

pub struct HtmlError;

impl IntoResponse for HtmlError {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        let html = r#"
        <div class="font-jetBrains container mx-auto p-4">
            <h1 class="text-4xl font-bold">Dont PANIC!!</h1>
            <p class="text-xl">An error occurred while processing your request.</p>
        </div>
        "#;
        Html(html).into_response()
    }
}

pub struct AppState {
    minijinja_enviroment: Environment<'static>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let mut env = Environment::new();
        env.add_template("base", include_str!("templates/base.html"))?;
        env.add_template("home", include_str!("templates/home.html"))?;
        env.add_template("error", include_str!("templates/error.html"))?;
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
