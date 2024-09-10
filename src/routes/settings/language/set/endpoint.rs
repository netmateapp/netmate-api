use std::sync::Arc;

use axum::{extract::State, Extension, Json};
use axum_macros::debug_handler;
use http::StatusCode;
use serde::Deserialize;
use tracing::info;

use crate::common::{id::AccountId, language::Language};

use super::{dsl::SetLanaguage, interpreter::SetLanguageImpl};


/*
pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<GetLanguageImpl>> {
    let get_language = GetLanguageImpl::try_new(db.clone()).await?;

    let login_session = LoginSessionLayer::try_new(db.clone(), cache.clone())
        .await
        .map_err(|e| InitError::<GetLanguageImpl>::new(e.into()))?;

    let services = ServiceBuilder::new()
        .layer(login_session);

    let router = Router::new()
        .route("/language", get(handler))
        .layer(services)
        .with_state(Arc::new(get_language));

    Ok(router)
} */


#[debug_handler]
pub async fn handler(
    State(routine): State<Arc<SetLanguageImpl>>,
    Extension(account_id): Extension<AccountId>,
    Json(payload): Json<Payload>,
) -> Result<(), StatusCode> {
    match routine.set_language(&account_id, &payload.language).await {
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