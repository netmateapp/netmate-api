use thiserror::Error;

use crate::common::{fallible::Fallible, profile::account_id::AccountId, tag::{non_top_tag::NonTopTagId, proposal_operation::ProposalOperation, relation::{validate_tag_relation, TagRelation}}};

pub(crate) trait GetTagRelationProposalOperation {
    async fn get_tag_relation_proposal_operation(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<Option<ProposalOperation>, GetTagRelationRatingError> {
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
            Err(e) => Err(GetTagRelationRatingError::GetTagRelationRatingFailed(e.into()))
        }
    }

    async fn fetch_tag_relation_proposal_operation(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<Option<ProposalOperation>, GetTagRelationRatingError>;

    async fn is_proposal_status_uncalculated(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<bool, GetTagRelationRatingError>;

    async fn delete_proposal_operation(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), GetTagRelationRatingError>;
}

#[derive(Debug, Error)]
pub enum GetTagRelationRatingError {
    #[error("タグ関係の評価の取得に失敗しました")]
    FetchTagRelationRatingOperationFailed(#[source] anyhow::Error),
    #[error("ステータスの取得に失敗しました")]
    IsStatusCalculatedFailed(#[source] anyhow::Error),
    #[error("提案者フラグの解除に失敗しました")]
    DeflagIsProposerFailed(#[source] anyhow::Error),
    #[error("タグ関係の評価の取得に失敗しました")]
    GetTagRelationRatingFailed(#[source] anyhow::Error)
}