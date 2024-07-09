use std::sync::Arc;

use axum::extract::State;
use axum::response::Html;
use minijinja::context;

use domain::register_event_goer;

use crate::app::AppState;
use crate::controllers::HtmlError;
use crate::extractors::LoggedUser;

pub async fn register(
    LoggedUser(user): LoggedUser,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, HtmlError> {
    let tmpl = state
        .get_minijinja_env()
        .get_template("components/future_meet_ups/register_button")?;
    register_event_goer(&state.database_gateway, &state.database_gateway, &user.id).await?;
    let context = context! {
        user => user,
        registered_user => true,
    };
    Ok(Html(tmpl.render(context)?))
}
