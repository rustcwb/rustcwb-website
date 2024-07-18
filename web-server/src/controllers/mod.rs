use axum::response::{Html, IntoResponse};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use domain::{Location, MeetUp, MeetUpState, User};

pub mod admin;
pub mod call_for_papers;
pub mod index;
pub mod meet_up;
pub mod meet_up_goers;
pub mod user;
pub mod voting;

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

impl<E> From<E> for HtmlError
where
    E: std::fmt::Display,
{
    fn from(err: E) -> Self {
        tracing::error!("Unexpected error: {}", err);
        HtmlError
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserPresenter {
    nickname: String,
    email: String,
}

impl From<User> for UserPresenter {
    fn from(user: User) -> Self {
        Self {
            nickname: user.nickname.chars().take(10).collect::<String>(),
            email: user.email,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MeetUpPresenter {
    id: Ulid,
    title: String,
    state: String,
    description: String,
    speaker: String,
    date: NaiveDate,
    link: String,
    location: Location,
}

impl From<MeetUp> for MeetUpPresenter {
    fn from(meetup: MeetUp) -> Self {
        let (state, title, description, speaker, link) = match meetup.state {
            MeetUpState::Done { paper, link } => (
                "Done".into(),
                paper.title,
                paper.description,
                paper.speaker,
                link.as_str().to_owned(),
            ),
            MeetUpState::Scheduled(paper) => (
                "Scheduled".into(),
                paper.title,
                paper.description,
                paper.speaker,
                String::new(),
            ),
            MeetUpState::CallForPapers => (
                "CallForPapers".into(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ),
            MeetUpState::Voting => (
                "Voting".into(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ),
        };
        Self {
            id: meetup.id,
            title,
            state,
            description,
            speaker,
            link,
            date: meetup.date,
            location: meetup.location,
        }
    }
}
