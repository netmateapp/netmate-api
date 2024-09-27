use std::{net::SocketAddr, sync::Arc};

use axum::{extract::{ConnectInfo, State}, http::StatusCode, response::{IntoResponse, Response}, routing::post, Json, Router};
use axum_macros::debug_handler;
use scylla::Session;
use serde::Serialize;
use tower::ServiceBuilder;
use tracing::info;

use crate::{common::{one_time_token::OneTimeToken, tag::top_tag::TopTagId}, helper::{error::InitError, middleware::{rate_limiter, session_starter, TimeUnit}, redis::Pool}};

use super::{dsl::{VerifyEmail, VerifyEmailError}, interpreter::VerifyEmailImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<VerifyEmailImpl>> {
    let services = ServiceBuilder::new()
        .layer(rate_limiter(db.clone(), cache.clone(), "vrfem", 3, 1, TimeUnit::HOURS).await?)
        .layer(session_starter(db.clone(), cache.clone()).await?);

    let verify_email = VerifyEmailImpl::try_new(db, cache).await?;

    let router = Router::new()
        .route("/verify_email", post(handler))
        .layer(services)
        .with_state(Arc::new(verify_email));

    Ok(router)
}

#[debug_handler]
pub async fn handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(routine): State<Arc<VerifyEmailImpl>>,
    Json(token): Json<OneTimeToken>
) -> Result<Response, StatusCode> {
    match routine.verify_email(&token).await {
        Ok((account_id, top_tag_id)) => {
            info!(
                ip_address = %addr.ip(),
                "メールアドレスの認証に成功しました。"
            );

            // セッション開始ミドルウェアにアカウントIDを渡す
            let mut response = Json(Body { top_tag_id }).into_response();
            response.extensions_mut().insert(account_id);

            Ok(response)
        },
        Err(e) => {
            info!(
                ip_address = %addr.ip(),
                error = %e,
                "メールアドレスの認証に失敗しました。"
            );

            match e {
                VerifyEmailError::OneTimeTokenAuthenticationFailed | VerifyEmailError::AccountAlreadyExists => Err(StatusCode::BAD_REQUEST),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

#[derive(Serialize)]
pub struct Body {
    top_tag_id: TopTagId,
}