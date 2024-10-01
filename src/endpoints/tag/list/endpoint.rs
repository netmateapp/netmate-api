use std::sync::Arc;

use axum::{extract::{Path, State}, response::{IntoResponse, Response}, routing::get, Json, Router};
use axum_macros::debug_handler;
use http::{header::{CACHE_CONTROL, ETAG, IF_NONE_MATCH}, HeaderMap, HeaderValue, StatusCode};
use scylla::Session;
use serde::Serialize;
use tower::ServiceBuilder;
use tracing::error;

use crate::{common::{page::ZeroBasedPage, tag::{hierarchy::TagHierarchy, tag_id::TagId, tag_info::TagInfo}}, helper::{cache::{check_if_none_match, create_etag}, error::InitError, middleware::rate_limiter, redis::connection::Pool}, middlewares::limit::TimeUnit};

use super::{dsl::ListRelatedTags, interpreter::ListRelatedTagsImpl};

pub async fn endpoint(db: Arc<Session>, cache: Arc<Pool>) -> Result<Router, InitError<ListRelatedTagsImpl>> {
    let services = ServiceBuilder::new()
        .layer(rate_limiter(db, cache.clone(), "lstrl", 90, 15, TimeUnit::MINS).await?);

    let interpreter = ListRelatedTagsImpl::try_new(cache).await?;

    let router = Router::new()
        .route("/tags", get(handler))
        .layer(services)
        .with_state(Arc::new(interpreter));

    Ok(router)
}

#[debug_handler]
pub async fn handler(
    State(routine): State<Arc<ListRelatedTagsImpl>>,
    Path((tag_id, relation, page, is_signed_in)): Path<(TagId, TagHierarchy, ZeroBasedPage, bool)>,
    headers: HeaderMap
) -> Result<Response, StatusCode> {
    match routine.list_related_tags(tag_id, relation, page).await {
        Ok(tags) => {
            if is_signed_in {
                const CACHE_CONTROL_VALUE: HeaderValue = HeaderValue::from_static("maxage=5");

                Ok((
                    [(CACHE_CONTROL, CACHE_CONTROL_VALUE)],
                    Json(Data { tags })
                ).into_response())
            } else {
                if let Some(if_none_match) = headers.get(IF_NONE_MATCH) {
                    if check_if_none_match(&to_bytes(&tags), if_none_match) {
                        return Ok(StatusCode::NOT_MODIFIED.into_response());
                    }
                }

                const CACHE_CONTROL_VALUE: HeaderValue = HeaderValue::from_static("s-maxage=1800, maxage=1800, immutable");

                Ok((
                    [(CACHE_CONTROL, CACHE_CONTROL_VALUE), (ETAG, create_etag(&to_bytes(&tags)))],
                    Json(Data { tags })
                ).into_response())
            }
        },
        Err(e) => {
            error!(
                error = %e,
                tag_id = %tag_id,
                relation = ?relation,
                page = %page,
                is_signed_in = %is_signed_in,
                "タグリストの取得に失敗しました"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Serialize)]
pub struct Data {
    tags: Vec<TagInfo>,
}

fn to_bytes(tags: &Vec<TagInfo>) -> Vec<u8> {
    let mut bytes = Vec::new();
    for tag in tags {
        bytes.extend(tag.id().value().value().as_bytes());
        bytes.extend(tag.name().value().as_bytes());
        let value: u8 = (u8::from(tag.is_proposal()) << 1) | u8::from(tag.is_stable());
        bytes.push(value);
    }
    bytes
}