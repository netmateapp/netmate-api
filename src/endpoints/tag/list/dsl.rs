use thiserror::Error;

use crate::common::{fallible::Fallible, page::ZeroBasedPage, tag::{relationship::TagRelationType, tag_id::TagId, tag_name::TagName}};

pub(crate) trait ListRelatedTags {
    async fn list_related_tags(&self, tag_id: TagId, relationship: TagRelationType, page: ZeroBasedPage) -> Fallible<Vec<TagInfo>, ListRelatedTagsError>;
}

#[derive(Debug, Error)]
pub enum ListRelatedTagsError {
    #[error("タグリストの取得に失敗しました")]
    ListRelatedTagsFailed(#[source] anyhow::Error),
}

pub struct TagInfo {
    id: TagId,
    name: TagName,
    is_unstable_proposal: bool,
}

impl TagInfo {
    pub fn new(id: TagId, name: TagName, is_unstable_proposal: bool) -> Self {
        TagInfo { id, name, is_unstable_proposal }
    }
}