use std::sync::Arc;

use axum::{extract::State, response::{IntoResponse, Response}, routing::get, Extension, Json, Router};
use axum_macros::debug_handler;
use http::{header::CACHE_CONTROL, HeaderValue, StatusCode};
use scylla::Session;
use serde::Serialize;
use tower::ServiceBuilder;
use tracing::error;

use crate::{common::{handle::{id::HandleId, share_count::HandleShareCount}, profile::account_id::AccountId}, helper::{error::InitError, middleware::{rate_limiter, session_manager}, redis::Pool}, middlewares::limit::TimeUnit};

use super::{dsl::CountHandlesShare, interpreter::CountHandlesShareImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<CountHandlesShareImpl>> {
    let services = ServiceBuilder::new()
        .layer(rate_limiter(db.clone(), cache.clone(), "cnths", 120, 1, TimeUnit::HOURS).await?)
        .layer(session_manager(db.clone(), cache).await?);

    let count_handles_share = CountHandlesShareImpl::try_new(db).await?;

    let router = Router::new()
        .route("/handles", get(handler))
        .layer(services)
        .with_state(Arc::new(count_handles_share));

    Ok(router)
}

#[debug_handler]
pub async fn handler(
    State(routine): State<Arc<CountHandlesShareImpl>>,
    Extension(account_id): Extension<AccountId>,
) -> Result<Response, StatusCode> {
    match routine.count_handles_share(account_id).await {
        Ok(handles) => {
            let handles = handles.into_iter()
                .map(|(handle_id, handle_share_count)| HandleInfo {
                    id: handle_id,
                    share_count: handle_share_count,
                })
                .collect();

            const CACHE_CONTROL_VALUE: HeaderValue = HeaderValue::from_static("private, max-age=60, must-revalidate");

            // ETagを追加

            Ok((
                [(CACHE_CONTROL, CACHE_CONTROL_VALUE)],
                Json(Body { handles })
            ).into_response())
        },
        Err(e) => {
            error!(
                error = %e,
                account_id = %account_id,
                "アカウントの名義の取得に失敗しました"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Serialize)]
pub struct Body {
    handles: Vec<HandleInfo>,
}

#[derive(Serialize)]
pub struct HandleInfo {
    id: HandleId,
    share_count: HandleShareCount,
}