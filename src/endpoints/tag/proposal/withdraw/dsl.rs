use thiserror::Error;

use crate::common::{fallible::Fallible, profile::account_id::AccountId, tag::{non_top_tag::NonTopTagId, relation::{validate_tag_relation, TagRelation}}};

pub(crate) trait WithdrawTagRelationProposal {
    async fn withdraw_tag_relation_proposal(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), WithdrawTagRelationProposalError> {
        match validate_tag_relation(subtag_id, supertag_id, relation) {
            Ok(()) => if self.is_proposer(account_id, subtag_id, supertag_id, relation).await? {
                if self.is_status_uncalculated(subtag_id, supertag_id, relation).await? {
                    self.withdraw(account_id, subtag_id, supertag_id, relation).await
                } else {
                    // ステータスが計算済みである場合は撤回できず、フラグも折る
                    self.delete_proposal_operation(account_id, subtag_id, supertag_id, relation).await?;
                    Err(WithdrawTagRelationProposalError::CannotWithdraw)
                }
            } else {
                Err(WithdrawTagRelationProposalError::NotProposer)
            },
            Err(e) => Err(WithdrawTagRelationProposalError::WithdrawTagRelationProposalFailed(e.into()))
        }
    }

    async fn is_proposer(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<bool, WithdrawTagRelationProposalError>;

    async fn is_status_uncalculated(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<bool, WithdrawTagRelationProposalError>;

    async fn delete_proposal_operation(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), WithdrawTagRelationProposalError>;

    async fn withdraw(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), WithdrawTagRelationProposalError>;
}

#[derive(Debug, Error)]
pub enum WithdrawTagRelationProposalError {
    #[error("提案者かどうかの確認に失敗しました")]
    IsProposerFailed(#[source] anyhow::Error),
    #[error("提案者ではないため撤回できません")]
    NotProposer,
    #[error("ステータスが計算済みかどうかの確認に失敗しました")]
    IsStatusUnalculatedFailed(#[source] anyhow::Error),
    #[error("この提案は撤回できません")]
    CannotWithdraw,
    #[error("提案者のフラグの削除に失敗しました")]
    DeflagIsProposerFailed(#[source] anyhow::Error),
    #[error("提案の削除に失敗しました")]
    WithdrawFailed(#[source] anyhow::Error),
    #[error("提案の撤回に失敗しました")]
    WithdrawTagRelationProposalFailed(#[source] anyhow::Error),
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use crate::{common::{fallible::Fallible, profile::account_id::AccountId, tag::{non_top_tag::NonTopTagId, relation::TagRelation}}, helper::test::mock_non_top_tag_id};

    use super::{WithdrawTagRelationProposal, WithdrawTagRelationProposalError};

    static IS_NOT_PROPOSER: LazyLock<NonTopTagId> = LazyLock::new(|| mock_non_top_tag_id(0));
    static CALCULATED: LazyLock<NonTopTagId> = LazyLock::new(|| mock_non_top_tag_id(1));
    static UNCALCULATED: LazyLock<NonTopTagId> = LazyLock::new(|| mock_non_top_tag_id(2));

    // 上位タグが下位タグより小さいとエラーになるため定数化
    static SUPERTAG: LazyLock<NonTopTagId> = LazyLock::new(|| mock_non_top_tag_id(3));

    struct MockWithdrawTagRelationProposal;

    impl WithdrawTagRelationProposal for MockWithdrawTagRelationProposal {
        async fn is_proposer(&self, _: AccountId, subtag_id: NonTopTagId, _: NonTopTagId, _: TagRelation) -> Fallible<bool, WithdrawTagRelationProposalError> {
            if subtag_id == *IS_NOT_PROPOSER {
                Ok(false)
            } else {
                Ok(true)
            }
        }

        async fn is_status_uncalculated(&self, subtag_id: NonTopTagId, _: NonTopTagId, _: TagRelation) -> Fallible<bool, WithdrawTagRelationProposalError> {
            if subtag_id == *CALCULATED {
                Ok(false)
            } else {
                Ok(true)
            }
        }
    
        async fn delete_proposal_operation(&self, _: AccountId, _: NonTopTagId, _: NonTopTagId, _: TagRelation) -> Fallible<(), WithdrawTagRelationProposalError> {
            Ok(())
        }
    
        async fn withdraw(&self, _: AccountId, _: NonTopTagId, _: NonTopTagId, _: TagRelation) -> Fallible<(), WithdrawTagRelationProposalError> {
            Ok(())
        }
    }

    async fn test_dsl(subtag_id: NonTopTagId) -> Fallible<(), WithdrawTagRelationProposalError> {
        MockWithdrawTagRelationProposal.withdraw_tag_relation_proposal(AccountId::gen(), subtag_id, *SUPERTAG, TagRelation::Inclusion).await
    }

    #[tokio::test]
    async fn is_not_proposer() {
        assert!(matches!(test_dsl(*IS_NOT_PROPOSER).await.err().unwrap(), WithdrawTagRelationProposalError::NotProposer))
    }

    #[tokio::test]
    async fn calculated() {
        assert!(matches!(test_dsl(*CALCULATED).await.err().unwrap(), WithdrawTagRelationProposalError::CannotWithdraw))
    }

    #[tokio::test]
    async fn withdraw() {
        assert!(test_dsl(*UNCALCULATED).await.is_ok());
    }
}