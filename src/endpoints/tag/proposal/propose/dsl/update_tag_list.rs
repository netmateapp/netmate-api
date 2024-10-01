use thiserror::Error;

use crate::common::{fallible::Fallible, tag::{non_top_tag::NonTopTagId, relation::TagRelation, tag_name::TagName}};

pub(crate) trait AddRelationToHierarchicalTagList {
    async fn add_relation_to_hierarchical_tag_list(&self, subtag_id: NonTopTagId, subtag_name: TagName, supertag_id: NonTopTagId, supertag_name: TagName, relation: TagRelation) -> Fallible<(), UpdateTagRelationListError> {
        match relation {
            TagRelation::Inclusion => self.add_inclusion_relation(subtag_id, subtag_name, supertag_id, supertag_name).await,
            TagRelation::Equivalence => self.add_equivalence_relation(subtag_id, subtag_name, supertag_id, supertag_name).await
        }
    }

    async fn add_inclusion_relation(&self, subtag_id: NonTopTagId, subtag_name: TagName, supertag_id: NonTopTagId, supertag_name: TagName) -> Fallible<(), UpdateTagRelationListError>;

    async fn add_equivalence_relation(&self, lesser_tag_id: NonTopTagId, lesser_tag_name: TagName, greater_tag_id: NonTopTagId, greater_tag_name: TagName) -> Fallible<(), UpdateTagRelationListError>;
}

#[derive(Debug, Error)]
pub enum UpdateTagRelationListError {
    #[error("階層別タグ一覧への包含関係の追加に失敗しました")]
    UpdateInclusionRelationListFailed(#[source] anyhow::Error),
    #[error("階層別タグ一覧への同値関係の追加に失敗しました")]
    UpdateEquivalenceRelationListFailed(#[source] anyhow::Error),
}