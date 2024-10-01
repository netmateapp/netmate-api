use thiserror::Error;

use crate::common::{fallible::Fallible, profile::account_id::AccountId, tag::{hierarchy::TagHierarchy, non_top_tag::NonTopTagId, rating_operation::RatingOperation, relation::{validate_tag_relation, TagRelation}}};

pub(crate) trait GetTagRelationRating {
    async fn get_tag_relation_rating(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<Option<RatingOperation>, GetTagRelationRatingError> {
        match validate_tag_relation(subtag_id, supertag_id, relation) {
            Ok(()) => match self.fetch_tag_relation_rating_operation(account_id, subtag_id, supertag_id, relation).await? {
                Some(RatingOperation::Proposed) => {
                    let hierarchy = match relation {
                        TagRelation::Inclusion => TagHierarchy::Super,
                        TagRelation::Equivalence => TagHierarchy::Equivalent
                    };
    
                    if self.is_status_calculated(subtag_id, supertag_id, hierarchy).await? {
                        self.deflag_is_proposer(account_id, subtag_id, supertag_id, relation).await?;
                        Ok(None)
                    } else {
                        Ok(Some(RatingOperation::Proposed))
                    }
                },
                v => Ok(v),
            },
            Err(e) => Err(GetTagRelationRatingError::GetTagRelationRatingFailed(e.into()))
        }
    }

    async fn fetch_tag_relation_rating_operation(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<Option<RatingOperation>, GetTagRelationRatingError>;

    // 提案フラグ取得後に実行されるため、Optionで包む必要はない
    async fn is_status_calculated(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, hierarchy: TagHierarchy) -> Fallible<bool, GetTagRelationRatingError>;

    async fn deflag_is_proposer(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), GetTagRelationRatingError>;
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