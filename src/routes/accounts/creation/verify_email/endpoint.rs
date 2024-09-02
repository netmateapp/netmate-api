use std::{net::SocketAddr, sync::Arc};

use axum::{extract::{ConnectInfo, State}, http::StatusCode, response::IntoResponse, Json, Router};
use deadpool_redis::Pool;
use scylla::Session;
use tower::ServiceBuilder;
use tracing::info;

use crate::{helper::error::InitError, middlewares::login_session::LoginSessionLayer, routes::accounts::creation::sign_up::value::OneTimeToken};

use super::{dsl::{VerifyEmail, VerifyEmailError}, interpreter::VerifyEmailImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<VerifyEmailImpl>> {
    let verify_email = VerifyEmailImpl::try_new(db.clone()).await?;

    let services = ServiceBuilder::new()
        .layer(LoginSessionLayer::new(db.clone(), cache.clone()));

    let router = Router::new()
        .layer(services)
        .with_state(Arc::new(verify_email));

    Ok(router)
}

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
