use thiserror::Error;

use crate::common::{fallible::Fallible, id::account_id::AccountId, rating::Rating, tag::{relation::{validate_tag_relation, TagRelation}, tag_id::TagId}};

pub(crate) trait RateTagRelation {
    async fn rate_tag_relation(&self, account_id: AccountId, subtag_id: TagId, supertag_id: TagId, relation: TagRelation, rating: Rating) -> Fallible<(), RateTagRelationError> {
        match validate_tag_relation(subtag_id, supertag_id, relation) {
            Ok(()) => {
                if !self.is_tag_relation_suggested(subtag_id, supertag_id, relation).await? {
                    Err(RateTagRelationError::UnsuggestedTagRelation)
                } else {
                    self.rate(account_id, subtag_id, supertag_id, relation, rating).await
                }
            },
            Err(e) => Err(RateTagRelationError::RateTagRelationFailed(e.into()))
        }
    }

    async fn is_tag_relation_suggested(&self, subtag_id: TagId, supertag_id: TagId, relation: TagRelation) -> Fallible<bool, RateTagRelationError>;

    async fn rate(&self, account_id: AccountId, subtag_id: TagId, supertag_id: TagId, relation: TagRelation, rating: Rating) -> Fallible<(), RateTagRelationError>;
}

#[derive(Debug, Error)]
pub enum RateTagRelationError {
    #[error("タグ関係が提案されているかの確認に失敗しました")]
    CheckSuggestedTagRelationFailed(#[source] anyhow::Error),
    #[error("提案されていないタグ関係です")]
    UnsuggestedTagRelation,
    #[error("タグ関係の評価に失敗しました")]
    RateTagRelationFailed(#[source] anyhow::Error)
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use crate::common::{fallible::Fallible, id::account_id::AccountId, rating::Rating, tag::{relation::TagRelation, tag_id::TagId}};

    use super::{RateTagRelation, RateTagRelationError};

    static UNSUGGESTED_RELATION_SUBTAG_ID: LazyLock<TagId> = LazyLock::new(TagId::gen);

    struct MockRateTagRelation;

    impl RateTagRelation for MockRateTagRelation {
        async fn is_tag_relation_suggested(&self, subtag_id: TagId, _: TagId, _: TagRelation) -> Fallible<bool, RateTagRelationError> {
            Ok(subtag_id != *UNSUGGESTED_RELATION_SUBTAG_ID)
        }

        async fn rate(&self, _: AccountId, _: TagId, _: TagId, _: TagRelation, _: Rating) -> Fallible<(), RateTagRelationError> {
            Ok(())
        }
    }

    async fn test_rate_tag_relation(subtag_id: TagId, supertag_id: TagId, relation: TagRelation) -> Fallible<(), RateTagRelationError> {
        MockRateTagRelation.rate_tag_relation(AccountId::gen(), subtag_id, supertag_id, relation, Rating::High).await
    }

    #[tokio::test]
    async fn check_suggested_tag_relation() {
        // 有効な提案の場合
        for relation in [TagRelation::Inclusion, TagRelation::Equivalence] {
            let res = test_rate_tag_relation(TagId::gen(), TagId::gen(), relation).await;
            assert!(res.is_ok());
        }

        // 無効な提案の場合
        for relation in [TagRelation::Inclusion, TagRelation::Equivalence] {
            let res = test_rate_tag_relation(*UNSUGGESTED_RELATION_SUBTAG_ID, TagId::gen(), relation).await;
            assert!(matches!(res.err().unwrap(), RateTagRelationError::UnsuggestedTagRelation));
        }
    }
}