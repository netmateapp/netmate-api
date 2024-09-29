use std::sync::Arc;

use axum::{extract::State, routing::post, Extension, Json, Router};
use axum_macros::debug_handler;
use http::StatusCode;
use scylla::Session;
use serde::Deserialize;
use tower::ServiceBuilder;
use tracing::info;

use crate::{common::profile::{account_id::AccountId, region::Region}, helper::{error::InitError, middleware::{rate_limiter, session_manager}, redis::connection::Pool}, middlewares::limit::TimeUnit};

use super::{dsl::SetRegion, interpreter::SetRegionImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<SetRegionImpl>> {
    let services = ServiceBuilder::new()
        .layer(rate_limiter(db.clone(), cache.clone(), "setrg", 5, 1, TimeUnit::HOURS).await?)
        .layer(session_manager(db.clone(), cache).await?);

    let set_region = SetRegionImpl::try_new(db).await?;

    let router = Router::new()
        .route("/region", post(handler))
        .layer(services)
        .with_state(Arc::new(set_region));

    Ok(router)
}

#[debug_handler]
pub async fn handler(
    State(routine): State<Arc<SetRegionImpl>>,
    Extension(account_id): Extension<AccountId>,
    Json(payload): Json<Payload>,
) -> Result<(), StatusCode> {
    match routine.set_region(account_id, payload.region).await {
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
    region: Region
}