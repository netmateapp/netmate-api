use std::sync::Arc;

use axum::{extract::{Path, State}, routing::post, Extension, Router};
use http::StatusCode;
use scylla::Session;
use tower::ServiceBuilder;
use tracing::error;

use crate::{common::{handle::id::HandleId, profile::account_id::AccountId}, helper::{error::InitError, middleware::{rate_limiter, session_manager}, redis::connection::Pool}, middlewares::limit::TimeUnit};

use super::{dsl::{DeleteHandle, DeleteHandleError}, interpreter::DeleteHandleImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<DeleteHandleImpl>> {
    let services = ServiceBuilder::new()
    .layer(rate_limiter(db.clone(), cache.clone(), "delhd", 10, 1, TimeUnit::HOURS).await?)
    .layer(session_manager(db.clone(), cache).await?);

    let delete_handle = DeleteHandleImpl::try_new(db).await?;

    let router = Router::new()
        .route("/handles/:id", post(handler))
        .layer(services)
        .with_state(Arc::new(delete_handle));

    Ok(router)
}

pub async fn handler(
    State(routine): State<Arc<DeleteHandleImpl>>,
    Extension(account_id): Extension<AccountId>,
    Path(handle_id): Path<HandleId>
) -> StatusCode {
    match routine.delete_handle_if_onymous(account_id, handle_id).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(DeleteHandleError::AnonymousHandle) => StatusCode::FORBIDDEN,
        Err(e) => {
            error!(
                error = %e,
                "名義の削除に失敗しました"
            );
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
