use std::sync::Arc;

use axum::{extract::State, routing::post, Extension, Json, Router};
use http::StatusCode;
use scylla::Session;
use serde::Deserialize;
use tower::ServiceBuilder;
use tracing::error;

use crate::{common::{profile::account_id::AccountId, tag::{non_top_tag::NonTopTagId, relation::TagRelation}}, helper::{error::InitError, middleware::{quota_limiter, rate_limiter, session_manager}, redis::connection::Pool}, middlewares::limit::TimeUnit};

use super::{dsl::propose::{ProposeTagRelation, ProposeTagRelationError}, interpreter::ProposeTagRelationImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<ProposeTagRelationImpl>> {
    let services = ServiceBuilder::new()
        .layer(rate_limiter(db.clone(), cache.clone(), "prtrl", 100, 15, TimeUnit::MINS).await?)
        .layer(session_manager(db.clone(), cache.clone()).await?)
        .layer(quota_limiter(db.clone(), cache.clone(), "prtrl", 1, TimeUnit::DAYS).await?);

    let interpreter = ProposeTagRelationImpl::try_new(db, cache).await?;

    let router = Router::new()
        .route("/proposals", post(handler))
        .layer(services)
        .with_state(Arc::new(interpreter));

    Ok(router)
}

pub async fn handler(
    State(routine): State<Arc<ProposeTagRelationImpl>>,
    Extension(account_id): Extension<AccountId>,
    Json(payload): Json<Payload>
) -> StatusCode {
    match routine.propose_tag_relation(account_id, payload.subtag_id, payload.supertag_id, payload.relation).await {
        Ok(()) => StatusCode::OK,
        Err(e) => match e {
            ProposeTagRelationError::InvalidTopology(_) | ProposeTagRelationError::HasAlreadyBeenProposed
            | ProposeTagRelationError::NonExistentTag | ProposeTagRelationError::DifferentLanguageGroups => StatusCode::BAD_REQUEST,
            _ => {
                error!(
                    error = %e,
                    account_id = %account_id,
                    subtag_id = %payload.subtag_id,
                    supertag_id = %payload.supertag_id,
                    relation = %payload.relation,
                    "タグ関係の提案に失敗しました"
                );
    
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

#[derive(Deserialize)]
pub struct Payload {
    subtag_id: NonTopTagId,
    supertag_id: NonTopTagId,
    relation: TagRelation,
}