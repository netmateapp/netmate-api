use thiserror::Error;

use crate::common::{fallible::Fallible, tag::{non_top_tag_id::NonTopTagId, relation::TagRelation}};

pub(crate) trait ValidateTopology {
    async fn validate_topology(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), ValidateTopologyError> {
        match relation {
            TagRelation::Inclusion => if !self.is_acyclic(subtag_id, supertag_id).await? {
                return Err(ValidateTopologyError::IsNotAcyclic);
            },
            TagRelation::Equivalence => if !self.is_equivalent(subtag_id, supertag_id).await? {
                return Err(ValidateTopologyError::IsNotEquivalent);
            }
        }
        Ok(())
    }

    async fn is_acyclic(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId) -> Fallible<bool, ValidateTopologyError>;

    async fn is_equivalent(&self, lesser_tag_id: NonTopTagId, greater_tag_id: NonTopTagId) -> Fallible<bool, ValidateTopologyError>;
}

#[derive(Debug, Error)]
pub enum ValidateTopologyError {
    #[error("非巡回性の判定に失敗しました")]
    IsAcyclicFailed(#[source] anyhow::Error),
    #[error("非巡回ではありません")]
    IsNotAcyclic,
    #[error("同値性の判定に失敗しました")]
    IsEquivalentFailed(#[source] anyhow::Error),
    #[error("同値ではありません")]
    IsNotEquivalent,
}