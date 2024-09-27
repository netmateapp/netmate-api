use std::sync::Arc;

use axum::{extract::State, routing::delete, Extension, Json, Router};
use http::StatusCode;
use scylla::Session;
use serde::Deserialize;
use tower::ServiceBuilder;
use tracing::error;

use crate::{common::{id::account_id::AccountId, tag::{non_top_tag_id::NonTopTagId, relation::TagRelation}}, helper::{error::InitError, middleware::{rate_limiter, session_manager, TimeUnit}, redis::Pool}};

use super::{dsl::{UnrateTagRelation, UnrateTagRelationError}, interpreter::UnrateTagRelationImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<UnrateTagRelationImpl>> {
    let services = ServiceBuilder::new()
    .layer(rate_limiter(db.clone(), cache.clone(), "urtrl", 150, 15, TimeUnit::MINS).await?)
    .layer(session_manager(db.clone(), cache).await?);

    let interpreter = UnrateTagRelationImpl::try_new(db).await?;

    let router = Router::new()
        .route("/rating", delete(handler))
        .layer(services)
        .with_state(Arc::new(interpreter));

    Ok(router)
}

pub async fn handler(
    State(routine): State<Arc<UnrateTagRelationImpl>>,
    Extension(account_id): Extension<AccountId>,
    Json(payload): Json<Payload>
) -> StatusCode {
    match routine.unrate_tag_relation(account_id, payload.subtag_id, payload.supertag_id, payload.inclusion_or_equivalence).await {
        Ok(()) => StatusCode::OK,
        Err(UnrateTagRelationError::UnrateTagRelationFailed(e)) => {
            error!(
                error = %e,
                account_id = %account_id,
                subtag_id = %payload.subtag_id,
                supertag_id = %payload.supertag_id,
                inclusion_or_equivalence = ?payload.inclusion_or_equivalence,
                "タグ関係の評価の取り消しに失敗しました"
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
}