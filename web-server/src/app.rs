use std::path::Path;
use std::sync::Arc;

use anyhow::{bail, Result};
use axum::routing::{get, post};
use axum::Router;
use minijinja::Environment;
use tower_http::services::ServeDir;

use gateway::github::GithubRestGateway;
use gateway::SqliteDatabaseGateway;

use crate::controllers::admin::admin_router;
use crate::controllers::call_for_papers::{call_for_papers, save_call_for_papers};
use crate::controllers::index::index;
use crate::controllers::meet_up::{meet_up, meet_up_metadata};
use crate::controllers::user::{github_login, logout, user};
use crate::controllers::voting::{paper_details, paper_no_details, store_vote, voting};

pub async fn build_app<T: Clone + Send + Sync + 'static>(
    assets_dir: impl AsRef<Path>,
    database_url: String,
    admin_details: (String, String),
    (client_id, client_secret): (String, String),
) -> Result<Router<T>> {
    Ok(Router::new()
        .route("/", get(index))
        .nest("/admin", admin_router())
        .route("/callForPapers", get(call_for_papers))
        .route("/callForPapers", post(save_call_for_papers))
        .route("/voting", get(voting))
        .route("/voting/paperDetails/:id", get(paper_details))
        .route("/voting/paperNoDetails/:id", get(paper_no_details))
        .route("/storeVote", post(store_vote))
        .route("/meetUp/:id", get(meet_up))
        .route("/meetUp/metadata/:id", get(meet_up_metadata))
        .route("/user", get(user))
        .route("/github/authorize", get(github_login))
        .route("/logout", get(logout))
        .with_state(Arc::new(AppState::new(
            SqliteDatabaseGateway::new(&database_url).await?,
            GithubRestGateway::new(client_id.clone(), client_secret),
            client_id,
            admin_details,
        )?))
        .fallback_service(ServeDir::new(assets_dir.as_ref())))
}

pub struct AppState {
    pub admin_details: (String, String),
    pub database_gateway: SqliteDatabaseGateway,
    pub github_gateway: GithubRestGateway,
    pub github_client_id: String,
    pub minijinja_enviroment: Environment<'static>,
}

macro_rules! add_template {
    ($env:ident, $path:literal) => {
        let Some(path) = $path
            .strip_prefix("templates/")
            .and_then(|path| path.strip_suffix(".html"))
        else {
            bail!("Invalid template path: {}", $path);
        };
        $env.add_template(path, include_str!($path))?;
    };
}

impl AppState {
    pub fn new(
        database_gateway: SqliteDatabaseGateway,
        github_gateway: GithubRestGateway,
        github_client_id: String,
        admin_details: (String, String),
    ) -> Result<Self> {
        let mut env = Environment::new();
        add_template!(env, "templates/base.html");
        add_template!(env, "templates/home.html");
        add_template!(env, "templates/admin.html");
        add_template!(env, "templates/user.html");
        add_template!(env, "templates/call_for_papers.html");
        add_template!(env, "templates/voting.html");
        add_template!(env, "templates/success.html");
        add_template!(env, "templates/components/vote_paper/paper.html");
        add_template!(env, "templates/components/vote_paper/paper_details.html");
        add_template!(env, "templates/components/past_meet_ups/past_meet_ups.html");
        add_template!(
            env,
            "templates/components/past_meet_ups/past_meet_up_metadata.html"
        );
        add_template!(env, "templates/components/past_meet_ups/past_meet_up.html");
        add_template!(
            env,
            "templates/components/admin/future_meet_up/future_meet_up.html"
        );
        add_template!(
            env,
            "templates/components/admin/future_meet_up/no_future_meet_up.html"
        );
        add_template!(
            env,
            "templates/components/admin/future_meet_up/call_for_papers.html"
        );
        add_template!(env, "templates/components/admin/future_meet_up/voting.html");
        add_template!(
            env,
            "templates/components/admin/future_meet_up/scheduled.html"
        );
        add_template!(
            env,
            "templates/components/future_meet_ups/future_meet_up.html"
        );
        add_template!(
            env,
            "templates/components/future_meet_ups/call_for_papers.html"
        );
        add_template!(env, "templates/components/future_meet_ups/voting.html");
        add_template!(env, "templates/components/future_meet_ups/scheduled.html");
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
