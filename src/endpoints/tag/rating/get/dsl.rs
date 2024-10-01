use thiserror::Error;

use crate::common::{fallible::Fallible, profile::account_id::AccountId, tag::{non_top_tag::NonTopTagId, proposal_operation::ProposalOperation, relation::{validate_tag_relation, TagRelation}}};

pub(crate) trait GetTagRelationProposalOperation {
    async fn get_tag_relation_proposal_operation(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<Option<ProposalOperation>, GetTagRelationProposalOperationError> {
        match validate_tag_relation(subtag_id, supertag_id, relation) {
            Ok(()) => match self.fetch_tag_relation_proposal_operation(account_id, subtag_id, supertag_id, relation).await? {
                Some(ProposalOperation::Proposed) => {
                    if self.is_proposal_status_uncalculated(subtag_id, supertag_id, relation).await? {
                        Ok(Some(ProposalOperation::Proposed))
                    } else {
                        self.delete_proposal_operation(account_id, subtag_id, supertag_id, relation).await?;
                        Ok(None)
                    }
                },
                v => Ok(v),
            },
            Err(e) => Err(GetTagRelationProposalOperationError::GetTagRelationRatingFailed(e.into()))
        }
    }

    async fn fetch_tag_relation_proposal_operation(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<Option<ProposalOperation>, GetTagRelationProposalOperationError>;

    async fn is_proposal_status_uncalculated(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<bool, GetTagRelationProposalOperationError>;

    async fn delete_proposal_operation(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), GetTagRelationProposalOperationError>;
}

#[derive(Debug, Error)]
pub enum GetTagRelationProposalOperationError {
    #[error("タグ関係の評価の取得に失敗しました")]
    FetchTagRelationRatingOperationFailed(#[source] anyhow::Error),
    #[error("ステータスが計算されたかどうかの確認に失敗しました")]
    IsStatusUncalculatedFailed(#[source] anyhow::Error),
    #[error("提案者フラグの解除に失敗しました")]
    DeflagIsProposerFailed(#[source] anyhow::Error),
    #[error("タグ関係の評価の取得に失敗しました")]
    GetTagRelationRatingFailed(#[source] anyhow::Error)
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use thiserror::Error;

    use crate::{common::{fallible::Fallible, profile::account_id::AccountId, tag::{non_top_tag::NonTopTagId, proposal_operation::ProposalOperation, relation::TagRelation, tag_id::TagId}, uuid::uuid4::Uuid4}, helper::test::{mock_non_top_tag_id, mock_uuid}};

    use super::{GetTagRelationProposalOperation, GetTagRelationProposalOperationError};

    struct MockGetTagRelationProposalOperation;

    #[derive(Debug, Error)]
    #[error("疑似エラー")]
    struct MockError;

    static RATED: LazyLock<NonTopTagId> = LazyLock::new(|| mock_non_top_tag_id(0));
    static CALCULATED: LazyLock<NonTopTagId> = LazyLock::new(|| mock_non_top_tag_id(1));
    static UNCALCULATED: LazyLock<NonTopTagId> = LazyLock::new(|| mock_non_top_tag_id(2));

    // 上位タグが下位タグより小さいとエラーになるため定数化
    static SUBTAG: LazyLock<NonTopTagId> = LazyLock::new(|| mock_non_top_tag_id(3));
    static SUPERTAG: LazyLock<NonTopTagId> = LazyLock::new(|| mock_non_top_tag_id(4));

    impl GetTagRelationProposalOperation for MockGetTagRelationProposalOperation {
        async fn fetch_tag_relation_proposal_operation(&self, _: AccountId, subtag_id: NonTopTagId, _: NonTopTagId, _: TagRelation) -> Fallible<Option<ProposalOperation>, GetTagRelationProposalOperationError> {
            if subtag_id == *CALCULATED || subtag_id == *UNCALCULATED {
                Ok(Some(ProposalOperation::Proposed))
            } else if subtag_id == *RATED {
                Ok(Some(ProposalOperation::HighRated))
            } else {
                Ok(None)
            }
        }

        async fn is_proposal_status_uncalculated(&self, subtag_id: NonTopTagId, _: NonTopTagId, _: TagRelation) -> Fallible<bool, GetTagRelationProposalOperationError> {
            if subtag_id == *CALCULATED {
                Ok(false)
            } else if subtag_id == *UNCALCULATED {
                Ok(true)
            } else {
                Err(GetTagRelationProposalOperationError::IsStatusUncalculatedFailed(MockError.into()))
            }
        }
    
        async fn delete_proposal_operation(&self, _: AccountId, _: NonTopTagId, _: NonTopTagId, _: TagRelation) -> Fallible<(), GetTagRelationProposalOperationError> {
            Ok(())
        }
    }

    async fn test_dsl(subtag_id: NonTopTagId) -> Fallible<Option<ProposalOperation>, GetTagRelationProposalOperationError> {
        MockGetTagRelationProposalOperation.get_tag_relation_proposal_operation(AccountId::gen(), subtag_id, *SUPERTAG, TagRelation::Inclusion).await
    }

    #[tokio::test]
    async fn rated() {
        assert_eq!(test_dsl(*RATED).await.unwrap(), Some(ProposalOperation::HighRated));
    }

    #[tokio::test]
    async fn calculated() {
        assert_eq!(test_dsl(*CALCULATED).await.unwrap(), None);
    }

    #[tokio::test]
    async fn uncalculated() {
        assert_eq!(test_dsl(*UNCALCULATED).await.unwrap(), Some(ProposalOperation::Proposed));
    }

    #[tokio::test]
    async fn other() {
        assert!(test_dsl(*SUBTAG).await.unwrap().is_none());
    }
}