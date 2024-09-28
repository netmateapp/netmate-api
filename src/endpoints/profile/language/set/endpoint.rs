use std::sync::Arc;

use axum::{extract::State, routing::post, Extension, Json, Router};
use axum_macros::debug_handler;
use http::StatusCode;
use scylla::Session;
use serde::Deserialize;
use tower::ServiceBuilder;
use tracing::info;

use crate::{common::{id::account_id::AccountId, language::Language}, helper::{error::InitError, middleware::{rate_limiter, session_manager}, redis::Pool}, middlewares::limit::TimeUnit};

use super::{dsl::SetLanaguage, interpreter::SetLanguageImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<SetLanguageImpl>> {
    let services = ServiceBuilder::new()
        .layer(rate_limiter(db.clone(), cache.clone(), "setln", 30, 1, TimeUnit::HOURS).await?)
        .layer(session_manager(db.clone(), cache).await?);

    let set_language = SetLanguageImpl::try_new(db).await?;

    let router = Router::new()
        .route("/language", post(handler))
        .layer(services)
        .with_state(Arc::new(set_language));

    Ok(router)
}

#[debug_handler]
pub async fn handler(
    State(routine): State<Arc<SetLanguageImpl>>,
    Extension(account_id): Extension<AccountId>,
    Json(payload): Json<Payload>,
) -> Result<(), StatusCode> {
    match routine.set_language(account_id, payload.language).await {
        Ok(()) => Ok(()),
        Err(e) => {
            info!(
                error = %e,
                "アカウントの言語設定を変更できませんでした。"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct Payload {
    language: Language
}