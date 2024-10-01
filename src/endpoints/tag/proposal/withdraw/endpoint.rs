use std::sync::Arc;

use axum::{extract::State, routing::delete, Extension, Json, Router};
use http::StatusCode;
use scylla::Session;
use serde::Deserialize;
use tower::ServiceBuilder;

use crate::{common::{profile::account_id::AccountId, tag::{non_top_tag::NonTopTagId, relation::TagRelation}}, helper::{error::InitError, middleware::{rate_limiter, session_manager}, redis::connection::Pool}, middlewares::limit::TimeUnit};

use super::{dsl::{WithdrawTagRelationProposal, WithdrawTagRelationProposalError}, interpreter::WithdrawTagRelationProposalImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<WithdrawTagRelationProposalImpl>> {
    let services = ServiceBuilder::new()
        .layer(rate_limiter(db.clone(), cache.clone(), "wttrl", 100, 1, TimeUnit::HOURS).await?)
        .layer(session_manager(db.clone(), cache.clone()).await?);

    let interpreter = WithdrawTagRelationProposalImpl::try_new(db, cache).await?;

    let router = Router::new()
        .route("/proposals", delete(handler))
        .layer(services)
        .with_state(Arc::new(interpreter));

    Ok(router)
}

pub async fn handler(
    State(routine): State<Arc<WithdrawTagRelationProposalImpl>>,
    Extension(account_id): Extension<AccountId>,
    Json(payload): Json<Payload>
) -> StatusCode {
    match routine.withdraw_tag_relation_proposal(account_id, payload.subtag_id, payload.supertag_id, payload.relation).await {
        Ok(()) => StatusCode::OK,
        Err(e) => match e {
            WithdrawTagRelationProposalError::NotProposer | WithdrawTagRelationProposalError::CannotWithdraw => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[derive(Deserialize)]
pub struct Payload {
    subtag_id: NonTopTagId,
    supertag_id: NonTopTagId,
    relation: TagRelation,
}