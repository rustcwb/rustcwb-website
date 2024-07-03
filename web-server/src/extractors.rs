use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{
        header::{AUTHORIZATION, WWW_AUTHENTICATE},
        request::Parts,
        HeaderMap, HeaderName, StatusCode,
    },
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use base64::{prelude::BASE64_STANDARD, Engine};

use domain::{login_with_access_token, User};

use crate::app::AppState;

pub struct AdminUser();

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AdminUser {
    type Rejection = (StatusCode, HeaderMap);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|header| {
                let header = header.to_str().ok()?;
                let encoded = header.strip_prefix("Basic ")?;
                let decoded = BASE64_STANDARD.decode(encoded).ok()?;
                let decoded = String::from_utf8_lossy(&decoded);
                let (user, pass) = decoded.split_once(':')?;
                if user == state.admin_details.0 && pass == state.admin_details.1 {
                    Some(AdminUser)
                } else {
                    None
                }
            })
            .ok_or((
                StatusCode::UNAUTHORIZED,
                headers_map(&[(WWW_AUTHENTICATE, "Basic realm=\"admin\", charset=\"UTF-8\"")])
                    .map_err(|err| {
                        tracing::error!("Error creating headers map: {err}");
                        (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new())
                    })?,
            ))?;
        Ok(AdminUser())
    }
}

#[derive(Debug)]
pub struct MaybeUser(pub Option<User>);

#[async_trait]
impl FromRequestParts<Arc<AppState>> for MaybeUser {
    type Rejection = (StatusCode, HeaderMap);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let cookie_jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new()))?;
        match cookie_jar.get("access_token") {
            Some(cookie) => {
                let access_token = cookie.value();
                let user = login_with_access_token(
                    &state.database_gateway,
                    &state.github_gateway,
                    access_token,
                )
                .await
                .ok();
                Ok(MaybeUser(user))
            }
            None => Ok(MaybeUser(None)),
        }
    }
}

#[derive(Debug)]
pub struct LoggedUser(pub User);

#[async_trait]
impl FromRequestParts<Arc<AppState>> for LoggedUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let user = MaybeUser::from_request_parts(parts, state)
            .await
            .map_err(|_| Redirect::to("/user").into_response())?
            .0
            .ok_or(Redirect::to("/user").into_response())?;
        Ok(Self(user))
    }
}

fn headers_map(headers: &[(HeaderName, &str)]) -> Result<HeaderMap> {
    let mut header_map = HeaderMap::new();
    for (header_name, value) in headers {
        header_map.insert(header_name.clone(), value.parse()?);
    }
    Ok(header_map)
}
