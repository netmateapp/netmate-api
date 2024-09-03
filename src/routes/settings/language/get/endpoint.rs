use std::sync::Arc;

use axum::{extract::{Request, State}, Json};
use axum_macros::debug_handler;
use http::StatusCode;
use tracing::info;

use crate::{common::{id::AccountId, language::Language}, routes::settings::language::get::dsl::GetLanguage};

use super::interpreter::GetLanguageImpl;



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
