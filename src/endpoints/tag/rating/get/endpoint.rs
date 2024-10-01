use std::sync::Arc;

use axum::{extract::{Path, State}, routing::get, Extension, Json, Router};
use axum_macros::debug_handler;
use http::StatusCode;
use scylla::Session;
use serde::Serialize;
use tower::ServiceBuilder;
use tracing::error;

use crate::{common::{profile::account_id::AccountId, tag::{non_top_tag::NonTopTagId, relation::TagRelation}}, helper::{error::InitError, middleware::{rate_limiter, session_manager}, redis::connection::Pool}, middlewares::limit::TimeUnit};

use super::{dsl::GetTagRelationProposalOperation, interpreter::GetTagRelationRatingImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<GetTagRelationRatingImpl>> {
    let services = ServiceBuilder::new()
    .layer(rate_limiter(db.clone(), cache.clone(), "gttrr", 300, 15, TimeUnit::HOURS).await?)
    .layer(session_manager(db.clone(), cache).await?);

    let interpreter = GetTagRelationRatingImpl::try_new(db).await?;

    let router = Router::new()
        .route("/ratings", get(handler))
        .layer(services)
        .with_state(Arc::new(interpreter));

    Ok(router)
}

#[debug_handler]
pub async fn handler(
    State(routine): State<Arc<GetTagRelationRatingImpl>>,
    Extension(account_id): Extension<AccountId>,
    Path((subtag_id, supertag_id, relation)): Path<(NonTopTagId, NonTopTagId, TagRelation)>
) -> Result<Json<Data>, StatusCode> {
    match routine.get_tag_relation_proposal_operation(account_id, subtag_id, supertag_id, relation).await {
        Ok(Some(operation)) => Ok(Json(Data { operation: Some(operation as u8) })),
        Ok(None) => Ok(Json(Data { operation: None })),
        Err(e) => {
            error!(
                error = %e,
                account_id = %account_id,
                subtag_id = %subtag_id,
                supertag_id = %supertag_id,
                relation = %relation,
                "タグ関係の提案への評価の取得に失敗しました"
            );

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}

#[derive(Serialize)]
pub struct Data {
    operation: Option<u8>,
}
