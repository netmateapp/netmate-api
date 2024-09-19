use std::sync::Arc;

use axum::{extract::State, response::{IntoResponse, Response}, routing::get, Extension, Json, Router};
use axum_macros::debug_handler;
use http::{header::CACHE_CONTROL, HeaderValue, StatusCode};
use scylla::Session;
use serde::Serialize;
use tower::ServiceBuilder;
use tracing::error;

use crate::{common::{handle::{id::HandleId, name::HandleName, share_count::HandleShareCount}, id::account_id::AccountId}, helper::{error::InitError, middleware::{rate_limiter, session_manager, TimeUnit}, redis::Pool}};

use super::{dsl::GetHandles, interpreter::GetHandlesImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<GetHandlesImpl>> {
    let services = ServiceBuilder::new()
        .layer(rate_limiter(db.clone(), cache.clone(), "lishd", 30, 1, TimeUnit::HOURS).await?)
        .layer(session_manager(db.clone(), cache).await?);

    let get_handles = GetHandlesImpl::try_new(db).await?;

    let router = Router::new()
        .route("/handles", get(handler))
        .layer(services)
        .with_state(Arc::new(get_handles));

    Ok(router)
}

#[debug_handler]
pub async fn handler(
    State(routine): State<Arc<GetHandlesImpl>>,
    Extension(account_id): Extension<AccountId>,
) -> Result<Response, StatusCode> {
    match routine.get_handles(account_id).await {
        Ok(handles) => {
            let handles = handles.into_iter()
                .map(|(handle_id, handle_name, handle_share_count)| Handle {
                    id: handle_id,
                    name: handle_name,
                    share_count: handle_share_count,
                })
                .collect();

            const CACHE_CONTROL_VALUE: HeaderValue = HeaderValue::from_static("private, max-age=3600, must-revalidate");

            // ETagを追加

            Ok((
                [(CACHE_CONTROL, CACHE_CONTROL_VALUE)],
                Json(Body { handles })
            ).into_response())
        },
        Err(e) => {
            error!(
                error = %e,
                "アカウントの名義の取得に失敗しました"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Serialize)]
pub struct Body {
    handles: Vec<Handle>,
}

#[derive(Serialize)]
pub struct Handle {
    id: HandleId,
    name: Option<HandleName>,
    share_count: HandleShareCount,
}