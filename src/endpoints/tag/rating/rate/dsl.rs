use thiserror::Error;

use crate::common::{cycle::Cycle, fallible::Fallible, profile::account_id::AccountId, rating::Rating, tag::{language_group::LanguageGroup, non_top_tag_id::NonTopTagId, relation::{validate_tag_relation, TagRelation}}};

pub(crate) trait RateTagRelation {
    async fn rate_tag_relation(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation, rating: Rating) -> Fallible<(), RateTagRelationError> {
        match validate_tag_relation(subtag_id, supertag_id, relation) {
            Ok(()) => {
                let (inclusion_or_equivalence, language_group) = self.fetch_tag_relation_proposed(subtag_id, supertag_id)
                    .await?
                    .ok_or_else(|| RateTagRelationError::NonProposedTagRelation)?;

                if relation == inclusion_or_equivalence {
                    self.rate(language_group, Cycle::current_cycle(), account_id, subtag_id, supertag_id, relation, rating).await
                } else {
                    Err(RateTagRelationError::NonProposedTagRelation)
                }
            },
            Err(e) => Err(RateTagRelationError::RateTagRelationFailed(e.into()))
        }
    }

    async fn fetch_tag_relation_proposed(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId) -> Fallible<Option<(TagRelation, LanguageGroup)>, RateTagRelationError>;

    async fn rate(&self, language_group: LanguageGroup, cycle: Cycle, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation, rating: Rating) -> Fallible<(), RateTagRelationError>;
}

#[derive(Debug, Error)]
pub enum RateTagRelationError {
    #[error("タグ関係が提案されているかの確認に失敗しました")]
    CheckProposedTagRelationFailed(#[source] anyhow::Error),
    #[error("提案されていないタグ関係です")]
    NonProposedTagRelation,
    #[error("タグ関係の評価に失敗しました")]
    RateTagRelationFailed(#[source] anyhow::Error)
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use uuid::Uuid;

    use crate::common::{cycle::Cycle, fallible::Fallible, profile::account_id::AccountId, rating::Rating, tag::{language_group::LanguageGroup, non_top_tag_id::NonTopTagId, relation::TagRelation, tag_id::TagId}, uuid::uuid4::Uuid4};

    use super::{RateTagRelation, RateTagRelationError};

    static NON_PROPOSED_RELATION_SUBTAG_ID: LazyLock<NonTopTagId> = LazyLock::new(|| {
        let uuid = Uuid::from_fields(0x01, 0x01, 0x4001, &[0x80, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x03]);
        NonTopTagId::try_from(TagId::of(Uuid4::new_unchecked(uuid))).unwrap()
    });

    struct MockRateTagRelation;

    impl RateTagRelation for MockRateTagRelation {
        async fn fetch_tag_relation_proposed(&self, _: NonTopTagId, supertag_id: NonTopTagId) -> Fallible<Option<(TagRelation, LanguageGroup)>, RateTagRelationError> {
            if supertag_id == *NON_PROPOSED_RELATION_SUBTAG_ID {
                Ok(None)
            } else {
                Ok(Some((TagRelation::Inclusion, LanguageGroup::Japanese)))
            }
        }

        async fn rate(&self, _: LanguageGroup, _: Cycle, _: AccountId, _: NonTopTagId, _: NonTopTagId, _: TagRelation, _: Rating) -> Fallible<(), RateTagRelationError> {
            Ok(())
        }
    }

    async fn test_rate_tag_relation(subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), RateTagRelationError> {
        MockRateTagRelation.rate_tag_relation(AccountId::gen(), subtag_id, supertag_id, relation, Rating::High).await
    }

    #[tokio::test]
    async fn check_proposed_tag_relation() {
        // 下位タグが上位タグより小さくなるよう設定
        let subtag_id = NonTopTagId::try_from(TagId::of(Uuid4::new_unchecked(Uuid::from_fields(0x01, 0x01, 0x4001, &[0x80, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01])))).unwrap();
        let supertag_id = NonTopTagId::try_from(TagId::of(Uuid4::new_unchecked(Uuid::from_fields(0x01, 0x01, 0x4001, &[0x80, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02])))).unwrap();

        // 有効な提案の場合
        for relation in [TagRelation::Inclusion, TagRelation::Equivalence] {
            let res = test_rate_tag_relation(subtag_id, supertag_id, relation).await;
            assert!(res.is_ok());
        }

        // 無効な提案の場合
        for relation in [TagRelation::Inclusion, TagRelation::Equivalence] {
            let res = test_rate_tag_relation(subtag_id, *NON_PROPOSED_RELATION_SUBTAG_ID, relation).await;
            assert!(matches!(res.err().unwrap(), RateTagRelationError::NonProposedTagRelation));
        }
    }
}