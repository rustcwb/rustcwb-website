use anyhow::anyhow;

use domain::{
    login_with_access_token, login_with_github_code, AccessToken, LoginMethod, RefreshTokenError,
};
use shared::utc_now;
use tests::{
    build_gateway, create_random_user, create_user_with_access_token_and_login_method,
    GithubGatewayMock,
};

#[::tokio::test]
async fn login_with_github_code_for_new_user() -> anyhow::Result<()> {
    let github_code = "some_code";
    let github_access_token = AccessToken::new("access_token_1".into(), utc_now());
    let github_refresh_token = AccessToken::new("refresh_access_token_1".into(), utc_now());
    let user_gateway = build_gateway().await?;
    let github_access_token_clone = github_access_token.clone();
    let github_gateway = GithubGatewayMock::default()
        .push_exchange_code(move |code| {
            assert_eq!(github_code, code);
            Ok((
                github_access_token_clone.clone(),
                github_refresh_token.clone(),
            ))
        })
        .await
        .push_user_info(move |access_token| {
            assert_eq!(&github_access_token, access_token);
            Ok(("nickname".into(), "email@email.com".into()))
        })
        .await;
    let user = login_with_github_code(&user_gateway, &github_gateway, github_code.into()).await?;
    assert_eq!("nickname", user.nickname);
    assert_eq!("email@email.com", user.email);
    assert_eq!(
        utc_now() + chrono::Duration::days(1),
        *user.access_token.expire_at()
    );
    Ok(())
}

#[::tokio::test]
async fn login_with_github_code_for_existent_user() -> anyhow::Result<()> {
    let github_code = "some_code";
    let github_access_token = AccessToken::new("access_token_1".into(), utc_now());
    let github_refresh_token = AccessToken::new("refresh_access_token_1".into(), utc_now());
    let user_gateway = build_gateway().await?;
    let user = create_random_user(&user_gateway).await?;
    let email = user.email.clone();
    let github_access_token_clone = github_access_token.clone();
    let github_gateway = GithubGatewayMock::default()
        .push_exchange_code(move |code| {
            assert_eq!(github_code, code);
            Ok((
                github_access_token_clone.clone(),
                github_refresh_token.clone(),
            ))
        })
        .await
        .push_user_info(move |access_token| {
            assert_eq!(&github_access_token, access_token);
            Ok(("nickname".into(), email.clone()))
        })
        .await;
    let logged_user =
        login_with_github_code(&user_gateway, &github_gateway, github_code.into()).await?;
    assert_eq!("nickname", logged_user.nickname);
    assert_eq!(user.email, logged_user.email);
    Ok(())
}

#[::tokio::test]
async fn login_with_access_token_for_not_expired_user() -> anyhow::Result<()> {
    let user_gateway = build_gateway().await?;
    let github_gateway = GithubGatewayMock::default();
    let user_1 = create_random_user(&user_gateway).await?;
    let user_2 = create_random_user(&user_gateway).await?;
    let logged_user_1 =
        login_with_access_token(&user_gateway, &github_gateway, user_1.access_token.token())
            .await?;
    let logged_user_2 =
        login_with_access_token(&user_gateway, &github_gateway, user_2.access_token.token())
            .await?;
    assert_eq!(user_1, logged_user_1);
    assert_eq!(user_2, logged_user_2);
    Ok(())
}

#[::tokio::test]
async fn login_with_expired_access_token_but_valid_github_access_token() -> anyhow::Result<()> {
    let user_gateway = build_gateway().await?;
    let github_access_token = AccessToken::generate_new();
    let github_access_token_clone = github_access_token.clone();
    let github_gateway = GithubGatewayMock::default()
        .push_user_info(move |access_token| {
            assert_eq!(access_token, &github_access_token_clone);
            Ok(("nickname".into(), "email@email.com".into()))
        })
        .await;
    let user = create_user_with_access_token_and_login_method(
        &user_gateway,
        AccessToken::new("token_1".into(), utc_now() - chrono::Duration::seconds(1)),
        LoginMethod::Github {
            access_token: github_access_token.clone(),
            refresh_token: AccessToken::generate_new(),
        },
    )
    .await?;

    let logged_user =
        login_with_access_token(&user_gateway, &github_gateway, user.access_token.token()).await?;
    assert_eq!("email@email.com", logged_user.email);
    assert_eq!("nickname", logged_user.nickname);
    assert_eq!(user.id, logged_user.id);
    assert_ne!(user.access_token, logged_user.access_token);
    Ok(())
}

#[::tokio::test]
async fn login_with_expired_access_token_expired_github_access_token_but_valid_refresh_token(
) -> anyhow::Result<()> {
    let user_gateway = build_gateway().await?;
    let github_access_token = AccessToken::generate_new();
    let github_refresh_token = AccessToken::generate_new();
    let github_access_token_clone = github_access_token.clone();
    let github_access_token_clone_2 = github_access_token.clone();
    let github_refresh_token_clone = github_refresh_token.clone();
    let github_gateway = GithubGatewayMock::default()
        .push_refresh_token(move |refresh_token| {
            assert_eq!(refresh_token, &github_refresh_token_clone);
            Ok((
                github_access_token_clone.clone(),
                AccessToken::generate_new(),
            ))
        })
        .await
        .push_user_info(move |access_token| {
            assert_eq!(access_token, &github_access_token_clone_2);
            Ok(("nickname".into(), "email@email.com".into()))
        })
        .await;
    let user = create_user_with_access_token_and_login_method(
        &user_gateway,
        AccessToken::new("token_1".into(), utc_now() - chrono::Duration::seconds(1)),
        LoginMethod::Github {
            access_token: AccessToken::new(
                "token_2".into(),
                utc_now() - chrono::Duration::seconds(1),
            )
            .clone(),
            refresh_token: github_refresh_token.clone(),
        },
    )
    .await?;

    let logged_user =
        login_with_access_token(&user_gateway, &github_gateway, user.access_token.token()).await?;
    assert_eq!("email@email.com", logged_user.email);
    assert_eq!("nickname", logged_user.nickname);
    assert_eq!(user.id, logged_user.id);
    let LoginMethod::Github {
        access_token,
        refresh_token: _,
    } = logged_user.login_method;
    assert_eq!(github_access_token, access_token);
    assert_ne!(user.access_token, logged_user.access_token);
    Ok(())
}

#[::tokio::test]
async fn login_with_all_expired_tokens() -> anyhow::Result<()> {
    let user_gateway = build_gateway().await?;
    let github_refresh_token = AccessToken::generate_new();
    let github_refresh_token_clone = github_refresh_token.clone();
    let github_gateway = GithubGatewayMock::default()
        .push_refresh_token(move |refresh_token| {
            assert_eq!(refresh_token, &github_refresh_token_clone);
            Err(RefreshTokenError::Unknown(anyhow!("Invalid token")))
        })
        .await;
    let user = create_user_with_access_token_and_login_method(
        &user_gateway,
        AccessToken::new("token_1".into(), utc_now() - chrono::Duration::seconds(1)),
        LoginMethod::Github {
            access_token: AccessToken::new(
                "token_2".into(),
                utc_now() - chrono::Duration::seconds(1),
            )
            .clone(),
            refresh_token: github_refresh_token.clone(),
        },
    )
    .await?;

    let err = login_with_access_token(&user_gateway, &github_gateway, user.access_token.token())
        .await
        .expect_err("Should return error");
    assert_eq!("Unknown error: `Invalid token`", err.to_string());
    Ok(())
}
