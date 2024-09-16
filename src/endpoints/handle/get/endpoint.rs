use std::sync::Arc;

use axum::{extract::State, routing::get, Extension, Json, Router};
use axum_macros::debug_handler;
use http::StatusCode;
use scylla::Session;
use serde::Serialize;
use tracing::error;

use crate::{common::{handle::{id::HandleId, name::HandleName, share_count::HandleShareCount}, id::account_id::AccountId}, helper::error::InitError};

use super::{dsl::GetHandles, interpreter::GetHandlesImpl};

pub async fn endpoint(db: Arc<Session>) -> Result<Router, InitError<GetHandlesImpl>> {
    let get_handles = GetHandlesImpl::try_new(db).await?;

    let router = Router::new()
        .route("/handles", get(handler))
        .with_state(Arc::new(get_handles));

    Ok(router)
}

#[debug_handler]
pub async fn handler(
    State(routine): State<Arc<GetHandlesImpl>>,
    Extension(account_id): Extension<AccountId>,
) -> Result<Json<Body>, StatusCode> {
    match routine.get_handles(account_id).await {
        Ok(handles) => {
            let handles = handles.into_iter()
                .map(|(handle_id, handle_name, handle_share_count)| Handle {
                    id: handle_id,
                    name: handle_name,
                    share_count: handle_share_count,
                })
                .collect();

            Ok(Json(Body { handles }))
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
    name: HandleName,
    share_count: HandleShareCount,
}