use std::sync::Arc;

use axum::{extract::State, routing::get, Extension, Json, Router};
use axum_macros::debug_handler;
use http::StatusCode;
use scylla::Session;
use tower::ServiceBuilder;
use tracing::info;

use crate::{common::{id::AccountId, language::Language}, helper::{error::InitError, middleware::{rate_limiter, session_manager}, valkey::Pool}, middlewares::rate_limit::dsl::increment_rate::{InculsiveLimit, TimeWindow}, routes::settings::language::get::dsl::GetLanguage};

use super::interpreter::GetLanguageImpl;

const ENDPOINT_NAME: &str = "getln";
const LIMIT: InculsiveLimit = InculsiveLimit::new(5);
const TIME_WINDOW: TimeWindow = TimeWindow::minutes(15);

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<GetLanguageImpl>> {
    let get_language = GetLanguageImpl::try_new(db.clone()).await?;

    let services = ServiceBuilder::new()
        .layer(rate_limiter(db.clone(), cache.clone(), ENDPOINT_NAME, LIMIT, TIME_WINDOW).await?)
        .layer(session_manager(db, cache).await?);

    let router = Router::new()
        .route("/language", get(handler))
        .layer(services)
        .with_state(Arc::new(get_language));

    Ok(router)
}


#[debug_handler]
pub async fn handler(
    State(routine): State<Arc<GetLanguageImpl>>,
    Extension(account_id): Extension<AccountId>,
) -> Result<Json<Language>, StatusCode> {
    match routine.get_language(&account_id).await {
        Ok(language) => Ok(Json(language)),
        Err(e) => {
            info!(
                error = %e,
                "アカウントの言語設定を取得できませんでした。"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
