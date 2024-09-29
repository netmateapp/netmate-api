use thiserror::Error;

use crate::common::{fallible::Fallible, profile::account_id::AccountId, tag::{non_top_tag::NonTopTagId, relation::TagRelation}};

pub(crate) trait GetTagRelationRating {
    async fn get_tag_relation_rating(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<RatingOperation, GetTagRelationRatingError> {
        Ok(RatingOperation::High)
    }
}

#[derive(Debug, Error)]
pub enum GetTagRelationRatingError {
    #[error("タグ関係の評価の取得に失敗しました")]
    GetTagRelationRatingFailed(#[source] anyhow::Error)
}

pub enum RatingOperation {
    Low,
    Middle,
    High,
    Proposed,
}