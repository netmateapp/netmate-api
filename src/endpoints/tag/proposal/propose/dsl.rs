use thiserror::Error;

use crate::common::{fallible::Fallible, tag::{non_top_tag_id::NonTopTagId, relation::{validate_tag_relation, TagRelation}}};

pub(crate) trait ProposeTagRelation {
    async fn propose_tag_relation(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), ProposeTagRelationError> {
        match validate_tag_relation(subtag_id, supertag_id, relation) {
            Ok(()) => match relation {
                TagRelation::Inclusion => {
                    if self.is_cycle_formed(subtag_id, supertag_id).await? {

                    } else {

                    }
                },
                TagRelation::Equivalence => {
                    
                }
            },
            Err(e) => () //ProposeTagRelationError::ProposeTagRelationFailed(e.into()),
        };

        Ok(())
    }

    async fn is_cycle_formed(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId) -> Fallible<bool, ProposeTagRelationError>;

    async fn is_equivalent(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId) -> Fallible<bool, ProposeTagRelationError>;

    async fn propose(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), ProposeTagRelationError>;
}

#[derive(Debug, Error)]
pub enum ProposeTagRelationError {
    #[error("サイクル検出の判定に失敗しました")]
    IsCycleFormedFailed(#[source] anyhow::Error),
    #[error("同値性の判定に失敗しました")]
    IsEquivalentFailed(#[source] anyhow::Error),
    #[error("タグ関係の提案に失敗しました")]
    ProposeTagRelationFailed(#[source] anyhow::Error),
}