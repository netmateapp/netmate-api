use std::sync::Arc;

use axum::{extract::State, response::{IntoResponse, Response}, routing::get, Extension, Json, Router};
use axum_macros::debug_handler;
use http::{header::{CACHE_CONTROL, ETAG}, HeaderMap, HeaderValue, StatusCode};
use scylla::Session;
use serde::Serialize;
use tower::ServiceBuilder;
use tracing::error;

use crate::{common::{handle::{id::HandleId, name::HandleName}, profile::account_id::AccountId}, helper::{cache::{check_if_none_match, create_etag}, error::InitError, middleware::{rate_limiter, session_manager}, redis::connection::Pool}, middlewares::limit::TimeUnit};

use super::{dsl::ListHandles, interpreter::ListHandlesImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<ListHandlesImpl>> {
    let services = ServiceBuilder::new()
        .layer(rate_limiter(db.clone(), cache.clone(), "lishd", 30, 1, TimeUnit::HOURS).await?)
        .layer(session_manager(db.clone(), cache).await?);

    let get_handles = ListHandlesImpl::try_new(db).await?;

    let router = Router::new()
        .route("/handles", get(handler))
        .layer(services)
        .with_state(Arc::new(get_handles));

    Ok(router)
}

#[debug_handler]
pub async fn handler(
    State(routine): State<Arc<ListHandlesImpl>>,
    Extension(account_id): Extension<AccountId>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    match routine.list_handles(account_id).await {
        Ok(handles) => {
            let handles = handles.into_iter()
                .map(|(handle_id, handle_name)| Handle {
                    id: handle_id,
                    name: handle_name,
                })
                .collect();

            if let Some(if_none_match) = headers.get("if-none-match") {
                if check_if_none_match(&to_bytes(&handles), if_none_match) {
                    return Ok(StatusCode::NOT_MODIFIED.into_response());
                }
            }

            const CACHE_CONTROL_VALUE: HeaderValue = HeaderValue::from_static("private, max-age=3600, must-revalidate");

            let etag_value = create_etag(&to_bytes(&handles));

            Ok((
                [(CACHE_CONTROL, CACHE_CONTROL_VALUE), (ETAG, etag_value)],
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

pub fn to_bytes(handles: &Vec<Handle>) -> Vec<u8> {
    let mut bytes = Vec::new();

    for handle in handles {
        bytes.extend(handle.id.value().value().to_bytes_le());
        if let Some(name) = &handle.name {
            bytes.extend(name.value().as_bytes());
        }
    }

    bytes
}

#[derive(Serialize)]
pub struct Handle {
    id: HandleId,
    name: Option<HandleName>,
}