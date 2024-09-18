use std::sync::Arc;

use axum::{extract::State, routing::post, Extension, Json, Router};
use http::StatusCode;
use scylla::Session;
use serde::Deserialize;
use tower::ServiceBuilder;
use tracing::error;

use crate::{common::{handle::name::NonAnonymousHandleName, id::account_id::AccountId}, helper::{error::InitError, middleware::{rate_limiter, session_manager, TimeUnit}, redis::Pool}};

use super::{dsl::CreateHandle, interpreter::CreateHandleImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<CreateHandleImpl>> {
    let services = ServiceBuilder::new()
    .layer(rate_limiter(db.clone(), cache.clone(), "crehd", 10, 1, TimeUnit::HOURS).await?)
    .layer(session_manager(db.clone(), cache).await?);

    let create_handle = CreateHandleImpl::try_new(db).await?;

    let router = Router::new()
        .route("/handles", post(handler))
        .layer(services)
        .with_state(Arc::new(create_handle));

    Ok(router)
}

pub async fn handler(
    State(routine): State<Arc<CreateHandleImpl>>,
    Extension(account_id): Extension<AccountId>,
    Json(payload): Json<Payload>
) -> StatusCode {
    match routine.create_handle(account_id, payload.handle_name).await {
        Ok(_) => StatusCode::CREATED,
        Err(e) => {
            error!(
                error = %e,
                "名義の作成に失敗しました"
            );
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[derive(Deserialize)]
pub struct Payload {
    handle_name: NonAnonymousHandleName
}