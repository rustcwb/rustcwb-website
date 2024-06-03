use anyhow::Result;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use axum::{extract::State, response::Html};
use axum_htmx::HxRequest;
use minijinja::context;
use minijinja::Environment;
use std::path::Path;
use std::sync::Arc;
use tower_http::services::ServeDir;

pub fn build_app<T: Clone + Send + Sync + 'static>(
    assets_dir: impl AsRef<Path>,
) -> Result<Router<T>> {
    Ok(Router::new()
        .route("/", get(index))
        .with_state(Arc::new(AppState::new()?))
        .fallback_service(ServeDir::new(assets_dir.as_ref())))
}

pub async fn index(
    HxRequest(is_hx_request): HxRequest,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state.get_minijinja_env().get_template("home")?;
    match is_hx_request {
        true => Ok(Html(
            tmpl.eval_to_state(context! {})?.render_block("content")?,
        )),
        false => Ok(Html(tmpl.render(context! {})?)),
    }
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
        // env.add_template("component/search_bar", include_str!("templates/Components/search_bar.html"))?;
        Ok(Self {
            minijinja_enviroment: env,
        })
    }

    pub fn get_minijinja_env(&self) -> &Environment<'static> {
        &self.minijinja_enviroment
    }
}
