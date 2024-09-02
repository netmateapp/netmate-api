use std::{net::SocketAddr, sync::Arc};

use axum::{error_handling::HandleErrorLayer, extract::{ConnectInfo, State}, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use axum_macros::debug_handler;
use bb8_redis::{bb8::Pool, RedisConnectionManager};
use scylla::Session;
use tower::ServiceBuilder;
use tracing::info;

use crate::{helper::error::InitError, middlewares::login_session::LoginSessionLayer, routes::accounts::creation::sign_up::value::OneTimeToken};

use super::{dsl::{VerifyEmail, VerifyEmailError}, interpreter::VerifyEmailImpl};

// cacheは使わない
pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool<RedisConnectionManager>>) -> Result<Router, InitError<VerifyEmailImpl>> {
    let verify_email = VerifyEmailImpl::try_new(db.clone()).await?;

    // 実際には`verify_email`では使わない
    let services = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: anyhow::Error| async move {
            StatusCode::BAD_REQUEST
        }))
        .layer(LoginSessionLayer::new(db.clone(), cache.clone()));

    let router = Router::new()
        .route("/verify_email", post(handler))
        .layer(services) // 実際には`verify_email`では使わない
        .with_state(Arc::new(verify_email));

    Ok(router)
}

#[debug_handler]
pub async fn handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(routine): State<Arc<VerifyEmailImpl>>,
    Json(token): Json<OneTimeToken>
) -> impl IntoResponse {
    match routine.verify_email(&token).await {
        Ok(language) => {
            info!(
                ip_address = %addr.ip(),
                "メールアドレスの認証に成功しました"
            );


            (StatusCode::OK, "")
        },
        Err(e) => {
            info!(
                ip_address = %addr.ip(),
                error = %e,
                "メールアドレスの認証に失敗しました"
            );

            match e {
                VerifyEmailError::OneTimeTokenAuthenticationFailed | VerifyEmailError::AccountAlreadyExists => (StatusCode::BAD_REQUEST, ""),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "")
            }
        }
    }
}
