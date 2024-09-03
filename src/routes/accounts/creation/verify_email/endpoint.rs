use std::{net::SocketAddr, sync::Arc};

use axum::{extract::{ConnectInfo, State}, http::StatusCode, routing::post, Json, Router};
use axum_macros::debug_handler;
use scylla::Session;
use tracing::info;

use crate::{helper::error::InitError, routes::accounts::creation::sign_up::value::OneTimeToken};

use super::{dsl::{VerifyEmail, VerifyEmailError}, interpreter::VerifyEmailImpl};

pub async fn endpoint(db: Arc<Session>) -> Result<Router, InitError<VerifyEmailImpl>> {
    let verify_email = VerifyEmailImpl::try_new(db).await?;

    let router = Router::new()
        .route("/verify_email", post(handler))
        .with_state(Arc::new(verify_email));

    Ok(router)
}

#[debug_handler]
pub async fn handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(routine): State<Arc<VerifyEmailImpl>>,
    Json(token): Json<OneTimeToken>
) -> Result<Json<String>, StatusCode> {
    match routine.verify_email(&token).await {
        Ok(top_tag_id) => {
            info!(
                ip_address = %addr.ip(),
                "メールアドレスの認証に成功しました"
            );

            Ok(Json(top_tag_id.value().to_string()))
        },
        Err(e) => {
            info!(
                ip_address = %addr.ip(),
                error = %e,
                "メールアドレスの認証に失敗しました"
            );

            match e {
                VerifyEmailError::OneTimeTokenAuthenticationFailed | VerifyEmailError::AccountAlreadyExists => Err(StatusCode::BAD_REQUEST),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}