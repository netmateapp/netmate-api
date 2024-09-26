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
        } else if !self.is_tag_relation_suggested(subtag_id, supertag_id, relation).await? {
            Err(RateTagRelationError::UnsuggestedTagRelation)
        } else {
            self.rate(account_id, subtag_id, supertag_id, relation, rating).await
        }
    }

    async fn is_tag_relation_suggested(&self, subtag_id: TagId, supertag_id: TagId, relation: TagRelation) -> Fallible<bool, RateTagRelationError>;

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

    use uuid::Uuid;

    use crate::common::{fallible::Fallible, id::account_id::AccountId, language::Language, rating::Rating, tag::{relation::TagRelation, tag_id::TagId, top_tag::top_tag_id_by_language}, uuid::uuid4::Uuid4};

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

    #[tokio::test]
    async fn compare_tags_in_equivalence_relation() {
        let subtag_id = TagId::of(Uuid4::new_unchecked(Uuid::from_fields(0x01, 0x01, 0x4001, &[0x80, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01])));
        let supertag_id = TagId::of(Uuid4::new_unchecked(Uuid::from_fields(0x01, 0x01, 0x4001, &[0x80, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02])));

        let res = test_rate_tag_relation(subtag_id, supertag_id, TagRelation::Equivalence).await;
        assert!(res.is_ok());

        // 下位タグと上位タグを逆転させる
        let res = test_rate_tag_relation(supertag_id, subtag_id, TagRelation::Equivalence).await;
        assert!(matches!(res.err().unwrap(), RateTagRelationError::SubtagIdMustBeSmallerThanSupertagIdInEquivalence));
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