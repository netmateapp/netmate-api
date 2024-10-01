use thiserror::Error;

use crate::common::{fallible::Fallible, tag::{non_top_tag::NonTopTagId, relation::TagRelation, tag_name::TagName}};

pub(crate) trait HierarchicalTagRelator {
    async fn relate_hierarchical_tags(&self, subtag_id: NonTopTagId, subtag_name: TagName, supertag_id: NonTopTagId, supertag_name: TagName, relation: TagRelation) -> Fallible<(), HierarchicalTagRelatorError> {
        match relation {
            TagRelation::Inclusion => self.relate_by_inclusion(subtag_id, subtag_name, supertag_id, supertag_name).await,
            TagRelation::Equivalence => self.relate_by_equivalence(subtag_id, subtag_name, supertag_id, supertag_name).await
        }
    }

    async fn relate_by_inclusion(&self, subtag_id: NonTopTagId, subtag_name: TagName, supertag_id: NonTopTagId, supertag_name: TagName) -> Fallible<(), HierarchicalTagRelatorError>;

    async fn relate_by_equivalence(&self, lesser_tag_id: NonTopTagId, lesser_tag_name: TagName, greater_tag_id: NonTopTagId, greater_tag_name: TagName) -> Fallible<(), HierarchicalTagRelatorError>;
}

#[derive(Debug, Error)]
pub enum HierarchicalTagRelatorError {
    #[error("階層別タグ一覧への包含関係の追加に失敗しました")]
    RelateByInclusionFailed(#[source] anyhow::Error),
    #[error("階層別タグ一覧への同値関係の追加に失敗しました")]
    RelateByEquivalenceFailed(#[source] anyhow::Error),
}