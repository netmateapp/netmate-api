use std::{net::SocketAddr, sync::Arc};

use axum::{extract::{ConnectInfo, State}, response::{IntoResponse, Response}, routing::post, Json, Router};
use http::StatusCode;
use scylla::Session;
use serde::Deserialize;
use tower::ServiceBuilder;
use tracing::info;

use crate::{common::{auth::password::Password, email::address::Email}, helper::{error::InitError, middleware::{rate_limiter, session_starter}, redis::connection::Pool}, middlewares::limit::TimeUnit};

use super::{dsl::SignIn, interpreter::SignInImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<SignInImpl>> {
    let services = ServiceBuilder::new()
        .layer(rate_limiter(db.clone(), cache.clone(), "sigin", 10, 1, TimeUnit::HOURS).await?)
        .layer(session_starter(db.clone(), cache).await?);

    let sign_in = SignInImpl::try_new(db).await?;

    let router = Router::new()
        .route("/sign_in", post(handler))
        .layer(services)
        .with_state(Arc::new(sign_in));

    Ok(router)
}

pub async fn handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(routine): State<Arc<SignInImpl>>,
    Json(payload): Json<Payload>,
) -> Result<Response, StatusCode> {
    match routine.sign_in(&payload.email, &payload.password).await {
        Ok(Some(account_id)) => {
            info!(
                ip_address = %addr.ip(),
                email = %payload.email,
                "ログインに成功しました。"
            );

            // セッション開始ミドルウェアにアカウントIDを渡す
            let mut response = StatusCode::OK.into_response();
            response.extensions_mut().insert(account_id);
            
            Ok(response)
        },
        Ok(None) => {
            info!(
                ip_address = %addr.ip(),
                email = %payload.email,
                "ログインに失敗しました。"
            );

            Err(StatusCode::BAD_REQUEST)
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct Payload {
    pub email: Email,
    pub password: Password,
}