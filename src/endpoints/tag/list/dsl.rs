use thiserror::Error;

use crate::common::{fallible::Fallible, page::ZeroBasedPage, tag::{hierarchy::TagHierarchy, tag_id::TagId, tag_info::TagInfo}};

pub(crate) trait ListRelatedTags {
    async fn list_related_tags(&self, tag_id: TagId, relationship: TagHierarchy, page: ZeroBasedPage) -> Fallible<Vec<TagInfo>, ListRelatedTagsError>;
}

#[derive(Debug, Error)]
pub enum ListRelatedTagsError {
    #[error("タグリストの取得に失敗しました")]
    ListRelatedTagsFailed(#[source] anyhow::Error),
}