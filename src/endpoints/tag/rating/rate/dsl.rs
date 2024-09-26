use thiserror::Error;

use crate::{common::{fallible::Fallible, id::account_id::AccountId, tag::{tag_id::TagId, top_tag::is_top_tag}}, endpoints::tag::rating::value::{Rating, TagRelationType}};

pub(crate) trait RateTagRelation {
    async fn rate_tag_relation(&self, account_id: AccountId, subtag_id: TagId, supertag_id: TagId, inclusion_or_equivalence: TagRelationType, rating: Rating) -> Fallible<(), RateTagRelationError> {
        if subtag_id == supertag_id {
            Err(RateTagRelationError::CannotRateSameTagRelation)
        } else if is_top_tag(subtag_id) || is_top_tag(supertag_id) {
            Err(RateTagRelationError::CannotRateTopTagRelation)
        } else if inclusion_or_equivalence == TagRelationType::Equivalence && subtag_id > supertag_id {
            Err(RateTagRelationError::SubtagIdMustBeSmallerThanSupertagIdInEquivalence)
        } else {
            self.rate(account_id, subtag_id, supertag_id, inclusion_or_equivalence, rating).await
        }
    }

    async fn rate(&self, account_id: AccountId, subtag_id: TagId, supertag_id: TagId, inclusion_or_equivalence: TagRelationType, rating: Rating) -> Fallible<(), RateTagRelationError>;
}

#[derive(Debug, Error)]
pub enum RateTagRelationError {
    #[error("同じタグ間の関係を評価することはできません")]
    CannotRateSameTagRelation,
    #[error("トップタグとの関係を評価することはできません")]
    CannotRateTopTagRelation,
    #[error("同値関係では`subtag_id`が`supertag_id`より小さくなければなりません")]
    SubtagIdMustBeSmallerThanSupertagIdInEquivalence,    
    #[error("タグ関係の評価に失敗しました")]
    RateTagRelationFailed(#[source] anyhow::Error)
}