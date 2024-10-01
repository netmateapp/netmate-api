use thiserror::Error;

use crate::common::{fallible::Fallible, tag::{non_top_tag::NonTopTagId, relation::TagRelation}};

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

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use crate::{common::{fallible::Fallible, tag::{non_top_tag::NonTopTagId, relation::TagRelation, tag_id::TagId}, uuid::uuid4::Uuid4}, helper::test::mock_uuid};

    use super::{ValidateTopology, ValidateTopologyError};

    struct MockValidateTopology;

    static VALID: LazyLock<NonTopTagId> = LazyLock::new(|| NonTopTagId::try_from(TagId::of(Uuid4::new_unchecked(mock_uuid(0)))).unwrap());
    static INVALID: LazyLock<NonTopTagId> = LazyLock::new(|| NonTopTagId::try_from(TagId::of(Uuid4::new_unchecked(mock_uuid(1)))).unwrap());

    impl ValidateTopology for MockValidateTopology {
        async fn is_acyclic(&self, subtag_id: NonTopTagId, _: NonTopTagId) -> Fallible<bool, ValidateTopologyError> {
            if subtag_id == *VALID {
                Ok(true)
            } else {
                Ok(false)
            }
        }

        async fn is_equivalent(&self, lesser_tag_id: NonTopTagId, _: NonTopTagId) -> Fallible<bool, ValidateTopologyError> {
            if lesser_tag_id == *VALID {
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    async fn test_dsl(subtag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), ValidateTopologyError> {
        MockValidateTopology.validate_topology(subtag_id, NonTopTagId::gen(), relation).await
    }

    #[tokio::test]
    async fn acyclic() {
        assert!(test_dsl(*VALID, TagRelation::Inclusion).await.is_ok());
    }

    #[tokio::test]
    async fn cyclic() {
        assert!(test_dsl(*INVALID, TagRelation::Inclusion).await.is_err());
    }

    #[tokio::test]
    async fn equivalent() {
        assert!(test_dsl(*VALID, TagRelation::Equivalence).await.is_ok());
    }

    #[tokio::test]
    async fn unequivalent() {
        assert!(test_dsl(*INVALID, TagRelation::Equivalence).await.is_err());
    }
}