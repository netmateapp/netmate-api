use std::sync::Arc;

use axum::{extract::State, routing::get, Extension, Json, Router};
use axum_macros::debug_handler;
use http::StatusCode;
use scylla::Session;
use tower::ServiceBuilder;
use tracing::info;

use crate::{common::{id::AccountId, language::Language}, helper::{error::InitError, valkey::Pool}, middlewares::manage_session::middleware::ManageSessionLayer, routes::settings::language::get::dsl::GetLanguage};

use super::interpreter::GetLanguageImpl;

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<GetLanguageImpl>> {
    let get_language = GetLanguageImpl::try_new(db.clone()).await?;

    let login_session = ManageSessionLayer::try_new(db.clone(), cache.clone())
        .await
        .map_err(|e| InitError::<GetLanguageImpl>::new(e.into()))?;

    let services = ServiceBuilder::new()
        .layer(login_session);

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
