use thiserror::Error;

use crate::common::{fallible::Fallible, id::account_id::AccountId, rating::Rating, tag::{relation::TagRelation, tag_id::TagId, top_tag::is_top_tag}};

pub(crate) trait RateTagRelation {
    async fn rate_tag_relation(&self, account_id: AccountId, subtag_id: TagId, supertag_id: TagId, relation: TagRelation, rating: Rating) -> Fallible<(), RateTagRelationError> {
        if subtag_id == supertag_id {
            Err(RateTagRelationError::CannotRateSameTagRelation)
        } else if is_top_tag(subtag_id) || is_top_tag(supertag_id) {
            Err(RateTagRelationError::CannotRateTopTagRelation)
        } else if relation == TagRelation::Equivalence && subtag_id > supertag_id {
            Err(RateTagRelationError::SubtagIdMustBeSmallerThanSupertagIdInEquivalence)
        } else if !self.is_relation_suggested(subtag_id, supertag_id, relation).await? {
            Err(RateTagRelationError::UnsuggestedTagRelation)
        } else {
            self.rate(account_id, subtag_id, supertag_id, relation, rating).await
        }
    }

    async fn is_relation_suggested(&self, subtag_id: TagId, supertag_id: TagId, relation: TagRelation) -> Fallible<bool, RateTagRelationError>;

    async fn rate(&self, account_id: AccountId, subtag_id: TagId, supertag_id: TagId, relation: TagRelation, rating: Rating) -> Fallible<(), RateTagRelationError>;
}

#[derive(Debug, Error)]
pub enum RateTagRelationError {
    #[error("同じタグ間の関係を評価することはできません")]
    CannotRateSameTagRelation,
    #[error("トップタグとの関係を評価することはできません")]
    CannotRateTopTagRelation,
    #[error("同値関係では`subtag_id`が`supertag_id`より小さくなければなりません")]
    SubtagIdMustBeSmallerThanSupertagIdInEquivalence,
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

    use crate::common::{fallible::Fallible, id::account_id::AccountId, language::Language, rating::Rating, tag::{relation::TagRelation, tag_id::TagId, top_tag::top_tag_id_by_language}};

    use super::{RateTagRelation, RateTagRelationError};

    static UNAVAILABLE_TAG_ID: LazyLock<TagId> = LazyLock::new(TagId::gen);

    struct MockRateTagRelation;

    impl RateTagRelation for MockRateTagRelation {
        async fn is_relation_suggested(&self, subtag_id: TagId, _: TagId, _: TagRelation) -> Fallible<bool, RateTagRelationError> {
            Ok(subtag_id != *UNAVAILABLE_TAG_ID)
        }

        async fn rate(&self, _: AccountId, _: TagId, _: TagId, _: TagRelation, _: Rating) -> Fallible<(), RateTagRelationError> {
            Ok(())
        }
    }

    async fn test_rate_tag_relation(subtag_id: TagId, supertag_id: TagId, relation: TagRelation) -> Fallible<(), RateTagRelationError> {
        MockRateTagRelation.rate_tag_relation(AccountId::gen(), subtag_id, supertag_id, relation, Rating::High).await
    }

    #[tokio::test]
    async fn same_tag() {
        let tag_id = TagId::gen();

        for relation in [TagRelation::Inclusion, TagRelation::Equivalence] {
            let res = test_rate_tag_relation(tag_id, tag_id, relation).await;
            assert!(matches!(res.err().unwrap(), RateTagRelationError::CannotRateSameTagRelation));
        }
    }

    #[tokio::test]
    async fn top_tag() {
        let top_tag_id = top_tag_id_by_language(Language::Japanese);

        for (subtag_id, supertag_id) in [(top_tag_id, TagId::gen()), (TagId::gen(), top_tag_id)] {
            for relation in [TagRelation::Inclusion, TagRelation::Equivalence] {
                let res = test_rate_tag_relation(subtag_id, supertag_id, relation).await;
                assert!(matches!(res.err().unwrap(), RateTagRelationError::CannotRateTopTagRelation));
            }
        }
    }
}