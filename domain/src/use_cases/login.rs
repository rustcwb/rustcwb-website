use anyhow::bail;
use ulid::Ulid;

use crate::{AccessToken, GetUserError, GithubGateway, LoginMethod, User, UserGateway};

pub async fn login_with_access_token(
    user_gateway: &impl UserGateway,
    github_gateway: &impl GithubGateway,
    access_token: &str,
) -> anyhow::Result<User> {
    let user = user_gateway.get_user_with_token(access_token).await?;
    if !user.access_token.is_expired() {
        return Ok(user);
    }
    match user.login_method {
        LoginMethod::Github {
            mut access_token,
            mut refresh_token,
        } => {
            if access_token.is_expired() {
                if refresh_token.is_expired() {
                    return Err(anyhow::anyhow!("Refresh token is expired"));
                }
                let (new_access_token, new_refresh_token) =
                    github_gateway.refresh_token(&refresh_token).await?;
                access_token = new_access_token;
                refresh_token = new_refresh_token;
            }
            let (nickname, email) = github_gateway.user_info(&access_token).await?;
            let user = user_gateway
                .store_user(User {
                    id: user.id,
                    access_token: AccessToken::generate_new(),
                    login_method: LoginMethod::Github {
                        access_token,
                        refresh_token,
                    },
                    nickname,
                    email,
                })
                .await?;
            Ok(user)
        }
    }
}

pub async fn login_with_github_code(
    user_gateway: &impl UserGateway,
    github_gateway: &impl GithubGateway,
    code: String,
) -> anyhow::Result<User> {
    let (access_token, refresh_token) = github_gateway.exchange_code(&code).await?;
    let (nickname, email) = github_gateway.user_info(&access_token).await?;
    // This code can have problems with concurrency. But this should be very unlikely
    let user = match user_gateway.get_user_with_email(&email).await {
        Ok(user) => {
            user_gateway
                .store_user(User {
                    id: user.id,
                    access_token: AccessToken::generate_new(),
                    login_method: LoginMethod::Github {
                        access_token,
                        refresh_token,
                    },
                    nickname,
                    email,
                })
                .await?
        }
        Err(GetUserError::NotFound) => {
            user_gateway
                .store_user(User {
                    id: Ulid::new(),
                    access_token: AccessToken::generate_new(),
                    login_method: LoginMethod::Github {
                        access_token,
                        refresh_token,
                    },
                    nickname,
                    email,
                })
                .await?
        }
        Err(err) => bail!("Error: {:?}", err),
    };
    Ok(user)
}
