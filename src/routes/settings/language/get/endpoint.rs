use std::sync::Arc;

use axum::{error_handling::HandleErrorLayer, extract::{Request, State}, routing::get, Json, Router};
use axum_macros::debug_handler;
use http::StatusCode;
use scylla::Session;
use tower::ServiceBuilder;
use tracing::info;

use crate::{common::{id::AccountId, language::Language}, helper::{error::InitError, garnet::Pool}, middlewares::session::{dsl::ManageSessionError, middleware::LoginSessionLayer}, routes::settings::language::get::dsl::GetLanguage};

use super::interpreter::GetLanguageImpl;

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<GetLanguageImpl>> {
    let get_language = GetLanguageImpl::try_new(db.clone()).await?;

    let login_session = LoginSessionLayer::try_new(db.clone(), cache.clone())
        .await
        .map_err(|e| InitError::<GetLanguageImpl>::new(e.into()))?;

    let services = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: ManageSessionError| async move { // ここは後から共通化
            StatusCode::BAD_REQUEST
        }))
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
    req: Request,
) -> Result<Json<Language>, (StatusCode, &'static str)> {
    // `Extensions`を使うと値が複製されるため、`Request`から直接取得する
    let account_id = req.extensions()
        .get::<AccountId>();

    match account_id {
        Some(account_id) => match routine.get_language(account_id).await {
            Ok(language) => Ok(Json(language)),
            Err(e) => {
                info!(
                    error = %e,
                    "アカウントの言語設定を取得できませんでした"
                );
                Err((StatusCode::INTERNAL_SERVER_ERROR, ""))
            }
        },
        None => { // ここに到達したら、認証ミドルウェアに不具合がある
            info!("アカウント識別子を取得できませんでした");
            Err((StatusCode::INTERNAL_SERVER_ERROR, ""))
        }
    }
}
