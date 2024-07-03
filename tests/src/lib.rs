use chrono::{NaiveDate, Utc};
use ulid::Ulid;

use domain::{
    AccessToken, LoginMethod, MeetUp, MeetUpGateway, MeetUpState, Paper, PaperGateway, User,
    UserGateway,
};
use gateway::SqliteDatabaseGateway;

pub async fn build_gateway() -> anyhow::Result<SqliteDatabaseGateway> {
    SqliteDatabaseGateway::new("sqlite::memory:").await
}

#[macro_export]
macro_rules! assert_meet_up_state {
    ($gateway:ident, $meet_up_id:expr, $state:pat) => {{
        let meet_up = ::domain::MeetUpGateway::get_meet_up(&$gateway, &$meet_up_id).await?;
        assert!(matches!(meet_up.state, $state))
    }};
}

pub async fn create_meet_up(
    gateway: &SqliteDatabaseGateway,
    location: String,
    date: NaiveDate,
    state: MeetUpState,
) -> anyhow::Result<MeetUp> {
    let meet_up = gateway.new_meet_up(Ulid::new(), location, date).await?;
    Ok(match state {
        MeetUpState::CallForPapers => meet_up,
        MeetUpState::Voting => gateway.update_meet_up_to_voting(&meet_up.id).await?,
        MeetUpState::Scheduled(paper) => {
            gateway.update_meet_up_to_voting(&meet_up.id).await?;
            gateway
                .store_paper_with_meet_up(&paper, &meet_up.id, 100)
                .await?;
            gateway
                .update_meet_up_to_scheduled(&meet_up.id, &paper.id)
                .await?
        }
        MeetUpState::Done { paper, link } => {
            gateway.update_meet_up_to_voting(&meet_up.id).await?;
            gateway
                .store_paper_with_meet_up(&paper, &meet_up.id, 100)
                .await?;
            gateway
                .update_meet_up_to_scheduled(&meet_up.id, &paper.id)
                .await?;
            gateway.finish_meet_up(&meet_up.id, link).await?;
            gateway.get_meet_up(&meet_up.id).await?
        }
    })
}

pub async fn create_random_user(gateway: &SqliteDatabaseGateway) -> anyhow::Result<User> {
    Ok(gateway
        .store_user(User {
            id: Ulid::new(),
            email: "email@email.com".into(),
            nickname: "nickname".into(),
            access_token: AccessToken::new("token".into(), Utc::now()),
            login_method: LoginMethod::Github {
                access_token: AccessToken::new("token".into(), Utc::now()),
                refresh_token: AccessToken::new("token".into(), Utc::now()),
            },
        })
        .await?)
}

pub fn build_paper_with_user(user_id: Ulid) -> Paper {
    Paper {
        id: Ulid::new(),
        user_id,
        title: "title".into(),
        description: "description".into(),
        speaker: "speaker".into(),
        email: "email".into(),
    }
}
