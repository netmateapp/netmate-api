use thiserror::Error;

use crate::common::{cycle::Cycle, fallible::Fallible, profile::account_id::AccountId, rating::Rating, tag::{language_group::LanguageGroup, non_top_tag::NonTopTagId, relation::{validate_tag_relation, TagRelation}}};

pub(crate) trait RateTagRelation {
    async fn rate_tag_relation(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation, rating: Rating) -> Fallible<(), RateTagRelationError> {
        match validate_tag_relation(subtag_id, supertag_id, relation) {
            Ok(()) => {
                let language_group = self.fetch_tag_relation_proposed(subtag_id, supertag_id, relation)
                    .await?
                    .ok_or_else(|| RateTagRelationError::NonProposedTagRelation)?;

                self.rate(language_group, Cycle::current_cycle(), account_id, subtag_id, supertag_id, relation, rating).await
            },
            Err(e) => Err(RateTagRelationError::RateTagRelationFailed(e.into()))
        }
    }

    async fn fetch_tag_relation_proposed(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<Option<LanguageGroup>, RateTagRelationError>;

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

    use crate::{common::{cycle::Cycle, fallible::Fallible, profile::account_id::AccountId, rating::Rating, tag::{language_group::LanguageGroup, non_top_tag::NonTopTagId, relation::TagRelation, tag_id::TagId}, uuid::uuid4::Uuid4}, helper::test::{mock_non_top_tag_id, mock_uuid}};

    use super::{RateTagRelation, RateTagRelationError};

    static NON_PROPOSED_RELATION_SUBTAG_ID: LazyLock<NonTopTagId> = LazyLock::new(|| mock_non_top_tag_id(3));

    struct MockRateTagRelation;

    impl RateTagRelation for MockRateTagRelation {
        async fn fetch_tag_relation_proposed(&self, _: NonTopTagId, supertag_id: NonTopTagId, _: TagRelation) -> Fallible<Option<LanguageGroup>, RateTagRelationError> {
            if supertag_id == *NON_PROPOSED_RELATION_SUBTAG_ID {
                Ok(None)
            } else {
                Ok(Some(LanguageGroup::Japanese))
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
        let subtag_id = mock_non_top_tag_id(1);
        let supertag_id = mock_non_top_tag_id(2);

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