use std::sync::Arc;

use axum::{extract::State, routing::get, Extension, Json, Router};
use axum_macros::debug_handler;
use http::StatusCode;
use scylla::Session;
use tower::ServiceBuilder;
use tracing::info;

use crate::{common::{id::account_id::AccountId, language::Language}, endpoints::profile::language::get::dsl::GetLanguage, helper::{error::InitError, middleware::{rate_limiter, session_manager, TimeUnit}, redis::Pool}};

use super::interpreter::GetLanguageImpl;

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<GetLanguageImpl>> {
    let services = ServiceBuilder::new()
        .layer(rate_limiter(db.clone(), cache.clone(), "getln", 5, 15, TimeUnit::MINS).await?)
        .layer(session_manager(db.clone(), cache).await?);

    let get_language = GetLanguageImpl::try_new(db).await?;

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
    match routine.get_language(account_id).await {
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
