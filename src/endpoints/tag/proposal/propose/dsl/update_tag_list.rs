use thiserror::Error;

use crate::common::{fallible::Fallible, tag::{non_top_tag::NonTopTagId, relation::TagRelation, tag_name::TagName}};

pub(crate) trait UpdateTagRelationList {
    async fn update_tag_relation_list(&self, subtag_id: NonTopTagId, subtag_name: TagName, supertag_id: NonTopTagId, supertag_name: TagName, relation: TagRelation) -> Fallible<(), UpdateTagRelationListError> {
        match relation {
            TagRelation::Inclusion => self.update_inclusion_relation_list(subtag_id, subtag_name, supertag_id, supertag_name).await,
            TagRelation::Equivalence => self.update_equivalence_relation_list(subtag_id, subtag_name, supertag_id, supertag_name).await
        }
    }

    async fn update_inclusion_relation_list(&self, subtag_id: NonTopTagId, subtag_name: TagName, supertag_id: NonTopTagId, supertag_name: TagName) -> Fallible<(), UpdateTagRelationListError>;

    async fn update_equivalence_relation_list(&self, lesser_tag_id: NonTopTagId, lesser_tag_name: TagName, greater_tag_id: NonTopTagId, greater_tag_name: TagName) -> Fallible<(), UpdateTagRelationListError>;
}

#[derive(Debug, Error)]
pub enum UpdateTagRelationListError {
    #[error("包含関係のタグ一覧の更新に失敗しました")]
    UpdateInclusionRelationListFailed(#[source] anyhow::Error),
    #[error("同値関係のタグ一覧の更新に失敗しました")]
    UpdateEquivalenceRelationListFailed(#[source] anyhow::Error),
}