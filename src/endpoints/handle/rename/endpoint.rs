use std::sync::Arc;

use axum::{extract::State, routing::patch, Extension, Json, Router};
use http::StatusCode;
use scylla::Session;
use serde::Deserialize;
use tower::ServiceBuilder;
use tracing::error;

use crate::{common::{handle::{id::HandleId, name::HandleName}, profile::account_id::AccountId}, helper::{error::InitError, middleware::{rate_limiter, session_manager}, redis::connection::Pool}, middlewares::limit::TimeUnit};

use super::{dsl::RenameHandle, interpreter::RenameHandleImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<RenameHandleImpl>> {
    let services = ServiceBuilder::new()
    .layer(rate_limiter(db.clone(), cache.clone(), "renhd", 30, 1, TimeUnit::HOURS).await?)
    .layer(session_manager(db.clone(), cache).await?);

    let rename_handle = RenameHandleImpl::try_new(db).await?;

    let router = Router::new()
        .route("/handles", patch(handler))
        .layer(services)
        .with_state(Arc::new(rename_handle));

    Ok(router)
}

pub async fn handler(
    State(routine): State<Arc<RenameHandleImpl>>,
    Extension(account_id): Extension<AccountId>,
    Json(payload): Json<Payload>
) -> StatusCode {
    match routine.rename_handle_if_onymous(account_id, payload.handle_id, payload.new_handle_name).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            error!(
                error = %e,
                "名義の編集に失敗しました"
            );
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[derive(Deserialize)]
pub struct Payload {
    handle_id: HandleId,
    new_handle_name: HandleName
}