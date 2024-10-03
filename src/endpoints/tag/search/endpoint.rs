use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use elasticsearch::Elasticsearch;
use http::StatusCode;
use scylla::Session;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tracing::error;

use crate::{common::tag::{hierarchy::TagHierarchy, language_group::LanguageGroup, tag_id::TagId, tag_name::TagName}, helper::{error::InitError, middleware::rate_limiter, redis::connection::Pool}, middlewares::limit::TimeUnit};

use super::{dsl::{SearchWithinHierarchicalTagList, TagInfo}, interpreter::SearchWithinHierarchicalTagListImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>, client: Arc<Elasticsearch>) -> Result<Router, InitError<SearchWithinHierarchicalTagListImpl>> {
    let services = ServiceBuilder::new()
        .layer(rate_limiter(db.clone(), cache, "srtrl", 300, 15, TimeUnit::MINS).await?);

    let interpreter = SearchWithinHierarchicalTagListImpl::try_new(db, client).await?;

    let router = Router::new()
        .route("/search", post(handler))
        .layer(services)
        .with_state(Arc::new(interpreter));

    Ok(router)
}

pub async fn handler(
    State(routine): State<Arc<SearchWithinHierarchicalTagListImpl>>,
    Json(payload): Json<Payload>
) -> Result<Json<Data>, StatusCode> {
    match routine.search_within_hierarchical_tag_list(&payload.query, payload.language_group, &payload.search_after, payload.tag_id, payload.hierarchy).await {
        Ok(tag_infos) => Ok(Json(Data { tags: tag_infos })),
        Err(e) => {
            error!(
                error = %e,
                query = %payload.query,
                language_group = %payload.language_group,
                search_after = ?payload.search_after,
                tag_id = %payload.tag_id,
                hierarchy = %payload.hierarchy,
                "階層別タグ一覧内の検索に失敗しました"
            );

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct Payload {
    query: TagName, // 部分的なタグ名
    tag_id: TagId,
    language_group: LanguageGroup,
    hierarchy: TagHierarchy,
    search_after: Option<TagId>
}

#[derive(Serialize)]
pub struct Data {
    tags: Vec<TagInfo>,
}