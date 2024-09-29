use std::{net::SocketAddr, sync::Arc};

use axum::{extract::{ConnectInfo, State}, routing::post, Extension, Router};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use axum_macros::debug_handler;
use http::StatusCode;
use scylla::Session;
use tower::ServiceBuilder;
use tracing::error;

use crate::{common::{profile::account_id::AccountId, session::{cookie::{REFRESH_PAIR_COOKIE_KEY, REFRESH_PAIR_SEPARATOR, SESSION_COOKIE_KEY}, session_series::SessionSeries}}, helper::{error::InitError, middleware::{rate_limiter, session_manager}, redis::connection::Pool}, middlewares::limit::TimeUnit};

use super::{dsl::SignOut, interpreter::SignOutImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<SignOutImpl>> {
    let services = ServiceBuilder::new()
        .layer(rate_limiter(db.clone(), cache.clone(), "sigot", 10, 1, TimeUnit::HOURS).await?)
        .layer(session_manager(db.clone(), cache.clone()).await?);

    let sign_out = SignOutImpl::try_new(db, cache).await?;

    let router = Router::new()
        .route("/sign_out", post(handler))
        .layer(services)
        .with_state(Arc::new(sign_out));

    Ok(router)
}

#[debug_handler]
async fn handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(routine): State<Arc<SignOutImpl>>,
    Extension(account_id): Extension<AccountId>,
    mut jar: CookieJar,
) -> Result<CookieJar, StatusCode> {
    let session_series: SessionSeries = extract_session_series(&jar)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // ログアウトの成否にかかわらず、クッキーを削除する
    jar = jar.remove(Cookie::build(SESSION_COOKIE_KEY));
    jar = jar.remove(Cookie::build(REFRESH_PAIR_COOKIE_KEY));

    match routine.sign_out(account_id, &session_series).await {
        Ok(_) => (),
        Err(e) => error!(
            error = %e,
            addr = %addr,
            account_id = %account_id,
            session_series = %session_series,
            "ログアウトに失敗しました。",
        )
    }

    Ok(jar)
}

// common/sessionと処理を共通化すべきでは
fn extract_session_series(jar: &CookieJar) -> Option<SessionSeries> {
    jar.get(REFRESH_PAIR_COOKIE_KEY)
        .and_then(|cookie| {
            cookie.value()
                .split(REFRESH_PAIR_SEPARATOR)
                .next()
                .and_then(|series| series.parse().ok())
        })
}