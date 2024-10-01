use thiserror::Error;

use crate::common::{fallible::Fallible, profile::account_id::AccountId, tag::{hierarchy::TagHierarchy, non_top_tag::NonTopTagId, relation::{validate_tag_relation, TagRelation}}};

pub(crate) trait WithdrawTagRelationProposal {
    async fn withdraw_tag_relation_proposal(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), WithdrawTagRelationProposalError> {
        match validate_tag_relation(subtag_id, supertag_id, relation) {
            Ok(()) => {
                if self.is_proposer(account_id, subtag_id, supertag_id, relation).await? {
                    let hierarchy = match relation {
                        TagRelation::Inclusion => TagHierarchy::Super,
                        TagRelation::Equivalence => TagHierarchy::Equivalent
                    };

                    // ステータスが計算済みであるのなら撤回できない
                    if self.is_status_calculated(subtag_id, supertag_id, hierarchy).await? {
                        self.deflag_is_proposer(account_id, subtag_id, supertag_id, relation).await?;
                        Err(WithdrawTagRelationProposalError::CannotWithdraw)
                    } else {
                        self.withdraw(account_id, subtag_id, supertag_id, relation).await
                    }
                } else {
                    Err(WithdrawTagRelationProposalError::NotProposer)
                }
            },
            Err(e) => Err(WithdrawTagRelationProposalError::WithdrawTagRelationProposalFailed(e.into()))
        }
    }

    async fn is_proposer(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<bool, WithdrawTagRelationProposalError>;

    async fn is_status_calculated(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, hierarchy: TagHierarchy) -> Fallible<bool, WithdrawTagRelationProposalError>;

    async fn deflag_is_proposer(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), WithdrawTagRelationProposalError>;

    async fn withdraw(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), WithdrawTagRelationProposalError>;
}

#[derive(Debug, Error)]
pub enum WithdrawTagRelationProposalError {
    #[error("提案者かどうかの確認に失敗しました")]
    IsProposerFailed(#[source] anyhow::Error),
    #[error("提案者ではないため撤回できません")]
    NotProposer,
    #[error("ステータスの確認に失敗しました")]
    IsStatusCalculatedFailed(#[source] anyhow::Error),
    #[error("この提案は撤回できません")]
    CannotWithdraw,
    #[error("提案者のフラグの削除に失敗しました")]
    DeflagIsProposerFailed(#[source] anyhow::Error),
    #[error("提案の削除に失敗しました")]
    WithdrawFailed(#[source] anyhow::Error),
    #[error("提案の撤回に失敗しました")]
    WithdrawTagRelationProposalFailed(#[source] anyhow::Error),
}