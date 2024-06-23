use std::sync::Arc;

use axum::{extract::State, response::Html, Form};
use markdown::Options;
use serde::Deserialize;

use crate::{app::AppState, extractors::LoggedUser};

pub async fn preview_markdown(
    _: LoggedUser,
    _: State<Arc<AppState>>,
    Form(params): Form<Params>,
) -> Html<String> {
    if params.description.is_empty() {
        return Html("".to_string());
    }
    match markdown::to_html_with_options(&params.description, &Options::default()) {
        Ok(html) => Html(format!(
            "<h1>Preview:</h1><div class=\"prose mt-4\">{html}</div>"
        )),
        Err(_) => Html("There is an error in your markdown".into()),
    }
}

#[derive(Debug, Deserialize)]
pub struct Params {
    description: String,
}
