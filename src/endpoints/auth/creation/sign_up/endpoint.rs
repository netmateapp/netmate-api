use std::{net::SocketAddr, sync::Arc};

use axum::{extract::{ConnectInfo, State}, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use scylla::Session;
use serde::Deserialize;
use tokio::task;
use tower::ServiceBuilder;
use tracing::info;

use crate::{common::{auth::password::Password, email::address::Email, profile::{birth_year::BirthYear, language::Language, region::Region}}, helper::middleware::rate_limiter, middlewares::limit::TimeUnit};
use crate::helper::{error::InitError, redis::connection::Pool};

use super::dsl::SignUp;
use super::interpreter::SignUpImpl;

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<SignUpImpl>> {
    let services = ServiceBuilder::new()
        .layer(rate_limiter(db.clone(), cache.clone(), "sigup", 5, 6, TimeUnit::HOURS).await?);

    let sign_up = SignUpImpl::try_new(db, cache).await?;

    let router = Router::new()
        .route("/sign_up", post(handler))
        .layer(services)
        .with_state(Arc::new(sign_up));

    Ok(router)
}

pub async fn handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(routine): State<Arc<SignUpImpl>>,
    Json(payload): Json<Payload>,
) -> impl IntoResponse {
    // 非 quick exit パターンを採用し、攻撃者に処理時間の差を計測させない
    task::spawn(async move {
        match routine.sign_up(&payload.email, &payload.password, payload.birth_year, payload.region, payload.language).await {
            // パスワードハッシュと生年は出力しない
            Ok(_) => info!(
                ip_address = %addr.ip(),
                email = %payload.email.value(),
                region = ?payload.region,
                language = ?payload.language,
                "アカウント作成の申請が正常に処理されました。"
            ),
            Err(e) => info!(
                ip_address = %addr.ip(),
                email = %payload.email.value(),
                region = ?payload.region,
                language = ?payload.language,
                error = %e,
                "アカウント作成の申請に失敗しました。"
            ),
        }
    });

    // `sign_up`の終了を待たずに返す
    StatusCode::OK
}

#[derive(Deserialize)]
pub struct Payload {
    pub email: Email,
    pub password: Password,
    pub region: Region,
    pub language: Language,
    pub birth_year: BirthYear,
}