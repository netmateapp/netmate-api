use thiserror::Error;

use crate::common::{fallible::Fallible, tag::{hierarchy::TagHierarchy, tag_id::TagId, tag_info::TagInfo}};

// search_afterを使う
pub(crate) trait SearchWithinHierarchicalTagList {
    async fn search_within_hierarchical_tag_list(&self, tag_id: TagId, related_tag_id: TagId, hierarchy: TagHierarchy) -> Fallible<Vec<TagInfo>, SearchWithinHierarchicalTagListError>;
}

#[derive(Debug, Error)]
pub enum SearchWithinHierarchicalTagListError {
    #[error("階層別タグ一覧内の検索に失敗しました")]
    SearchWithinHierarchicalTagListFailed(#[source] anyhow::Error),
}