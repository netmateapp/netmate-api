use thiserror::Error;

use crate::common::{fallible::Fallible, id::account_id::AccountId, tag::{relation::TagRelation, tag_id::TagId, top_tag::is_top_tag}};

pub(crate) trait UnrateTagRelation {
    async fn unrate_tag_relation(&self, account_id: AccountId, subtag_id: TagId, supertag_id: TagId, relation: TagRelation) -> Fallible<(), UnrateTagRelationError> {
        // DBを参照せずとも弾けるものは弾く
        if subtag_id == supertag_id {
            Err(UnrateTagRelationError::CannotRateSameTagRelation)
        } else if is_top_tag(subtag_id) || is_top_tag(supertag_id) {
            Err(UnrateTagRelationError::CannotRateTopTagRelation)
        } else if relation == TagRelation::Equivalence && subtag_id > supertag_id {
            Err(UnrateTagRelationError::SubtagIdMustBeSmallerThanSupertagIdInEquivalence)
        // DBを参照して弾く
        } else if !self.is_tag_relation_suggested(subtag_id, supertag_id, relation).await? {
            Err(UnrateTagRelationError::UnsuggestedTagRelation)
        } else {
            self.unrate(account_id, subtag_id, supertag_id, relation).await
        }
    }

    async fn is_tag_relation_suggested(&self, subtag_id: TagId, supertag_id: TagId, relation: TagRelation) -> Fallible<bool, UnrateTagRelationError>;

    async fn unrate(&self, account_id: AccountId, subtag_id: TagId, supertag_id: TagId, relation: TagRelation) -> Fallible<(), UnrateTagRelationError>;
}

#[derive(Debug, Error)]
pub enum UnrateTagRelationError {
    #[error("同じタグ間の関係の評価を取り消すことはできません")]
    CannotRateSameTagRelation,
    #[error("トップタグとの関係の評価を取り消すことはできません")]
    CannotRateTopTagRelation,
    #[error("同値関係では`subtag_id`が`supertag_id`より小さくなければなりません")]
    SubtagIdMustBeSmallerThanSupertagIdInEquivalence,
    #[error("タグ関係が提案されているかの確認に失敗しました")]
    CheckSuggestedTagRelationFailed(#[source] anyhow::Error),
    #[error("提案されていないタグ関係です")]
    UnsuggestedTagRelation,
    #[error("タグ関係への評価の取り消しに失敗しました")]
    UnrateTagRelationFailed(#[source] anyhow::Error)
}