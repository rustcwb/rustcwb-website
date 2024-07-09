use chrono::NaiveDate;
use fake::faker::internet::en::FreeEmail;
use fake::faker::name::en::FirstName;
use fake::Fake;
use tokio::sync::Mutex;
use ulid::Ulid;

use domain::{
    AccessToken, ExchangeCodeError, GithubGateway, LoginMethod, MeetUp, MeetUpGateway, MeetUpState,
    Paper, PaperGateway, RefreshTokenError, User, UserGateway, UserInfoGithubError,
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
            email: FreeEmail().fake(),
            nickname: FirstName().fake(),
            access_token: AccessToken::generate_new(),
            login_method: LoginMethod::Github {
                access_token: AccessToken::generate_new(),
                refresh_token: AccessToken::generate_new(),
            },
        })
        .await?)
}

pub async fn create_user_with_access_token_and_login_method(
    gateway: &SqliteDatabaseGateway,
    access_token: AccessToken,
    login_method: LoginMethod,
) -> anyhow::Result<User> {
    Ok(gateway
        .store_user(User {
            id: Ulid::new(),
            email: FreeEmail().fake(),
            nickname: FirstName().fake(),
            access_token,
            login_method,
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

pub struct GithubGatewayMock {
    internal: Mutex<InternalGithubGatewayMock>,
}

type UserInfoCallable = dyn Fn(&AccessToken) -> Result<(String, String), UserInfoGithubError>;
type RefreshTokenCallable =
    dyn Fn(&AccessToken) -> Result<(AccessToken, AccessToken), RefreshTokenError>;
type ExchangeCodeCallable = dyn Fn(&str) -> Result<(AccessToken, AccessToken), ExchangeCodeError>;

struct InternalGithubGatewayMock {
    user_infos: Vec<Box<UserInfoCallable>>,
    refresh_tokens: Vec<Box<RefreshTokenCallable>>,
    exchange_codes: Vec<Box<ExchangeCodeCallable>>,
}

impl Default for GithubGatewayMock {
    fn default() -> Self {
        Self {
            internal: Mutex::new(InternalGithubGatewayMock {
                user_infos: Vec::new(),
                refresh_tokens: Vec::new(),
                exchange_codes: Vec::new(),
            }),
        }
    }
}

impl GithubGatewayMock {
    pub async fn push_user_info(
        self,
        callable: impl Fn(&AccessToken) -> Result<(String, String), UserInfoGithubError> + 'static,
    ) -> Self {
        self.internal
            .lock()
            .await
            .user_infos
            .push(Box::new(callable));
        self
    }

    pub async fn push_refresh_token(
        self,
        callable: impl Fn(&AccessToken) -> Result<(AccessToken, AccessToken), RefreshTokenError>
            + 'static,
    ) -> Self {
        self.internal
            .lock()
            .await
            .refresh_tokens
            .push(Box::new(callable));
        self
    }

    pub async fn push_exchange_code(
        self,
        callable: impl Fn(&str) -> Result<(AccessToken, AccessToken), ExchangeCodeError> + 'static,
    ) -> Self {
        self.internal
            .lock()
            .await
            .exchange_codes
            .push(Box::new(callable));
        self
    }
}

impl GithubGateway for GithubGatewayMock {
    async fn user_info(
        &self,
        access_token: &AccessToken,
    ) -> Result<(String, String), UserInfoGithubError> {
        self.internal.lock().await.user_infos.pop().unwrap()(access_token)
    }

    async fn refresh_token(
        &self,
        refresh_token: &AccessToken,
    ) -> Result<(AccessToken, AccessToken), RefreshTokenError> {
        self.internal.lock().await.refresh_tokens.pop().unwrap()(refresh_token)
    }

    async fn exchange_code(
        &self,
        code: &str,
    ) -> Result<(AccessToken, AccessToken), ExchangeCodeError> {
        self.internal.lock().await.exchange_codes.pop().unwrap()(code)
    }
}
