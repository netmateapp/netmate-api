use std::sync::Arc;

use axum::{extract::State, routing::put, Extension, Json, Router};
use http::StatusCode;
use scylla::Session;
use serde::Deserialize;
use tower::ServiceBuilder;
use tracing::error;

use crate::{common::{profile::account_id::AccountId, rating::Rating, tag::{non_top_tag::NonTopTagId, relation::TagRelation}}, helper::{error::InitError, middleware::{rate_limiter, session_manager}, redis::connection::Pool}, middlewares::limit::TimeUnit};

use super::{dsl::{RateTagRelation, RateTagRelationError}, interpreter::RateTagRelationImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<RateTagRelationImpl>> {
    let services = ServiceBuilder::new()
    .layer(rate_limiter(db.clone(), cache.clone(), "rttrl", 150, 15, TimeUnit::MINS).await?)
    .layer(session_manager(db.clone(), cache).await?);

    let interpreter = RateTagRelationImpl::try_new(db).await?;

    let router = Router::new()
        .route("/ratings", put(handler))
        .layer(services)
        .with_state(Arc::new(interpreter));

    Ok(router)
}

pub async fn handler(
    State(routine): State<Arc<RateTagRelationImpl>>,
    Extension(account_id): Extension<AccountId>,
    Json(payload): Json<Payload>
) -> StatusCode {
    match routine.rate_tag_relation(account_id, payload.subtag_id, payload.supertag_id, payload.inclusion_or_equivalence, payload.rating).await {
        Ok(()) => StatusCode::OK,
        Err(RateTagRelationError::RateTagRelationFailed(e)) => {
            error!(
                error = %e,
                account_id = %account_id,
                subtag_id = %payload.subtag_id,
                supertag_id = %payload.supertag_id,
                inclusion_or_equivalence = ?payload.inclusion_or_equivalence,
                rating = ?payload.rating,
                "タグ関係の評価に失敗しました"
            );
            StatusCode::INTERNAL_SERVER_ERROR
        },
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

#[derive(Deserialize)]
pub struct Payload {
    subtag_id: NonTopTagId,
    supertag_id: NonTopTagId,
    inclusion_or_equivalence: TagRelation,
    rating: Rating,
}