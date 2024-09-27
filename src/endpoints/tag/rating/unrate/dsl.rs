use thiserror::Error;

use crate::common::{fallible::Fallible, id::account_id::AccountId, tag::{non_top_tag_id::NonTopTagId, relation::{validate_tag_relation, TagRelation}}};

pub(crate) trait UnrateTagRelation {
    async fn unrate_tag_relation(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), UnrateTagRelationError> {
        match validate_tag_relation(subtag_id, supertag_id, relation) {
            Ok(()) => {
                if !self.is_tag_relation_proposed(subtag_id, supertag_id, relation).await? {
                    Err(UnrateTagRelationError::NonProposedTagRelation)
                } else {
                    self.unrate(account_id, subtag_id, supertag_id, relation).await
                }
            },
            Err(e) => Err(UnrateTagRelationError::UnrateTagRelationFailed(e.into()))
        }
    }

    async fn is_tag_relation_proposed(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<bool, UnrateTagRelationError>;

    async fn unrate(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), UnrateTagRelationError>;
}

#[derive(Debug, Error)]
pub enum UnrateTagRelationError {
    #[error("タグ関係が提案されているかの確認に失敗しました")]
    CheckProposedTagRelationFailed(#[source] anyhow::Error),
    #[error("提案されていないタグ関係です")]
    NonProposedTagRelation,
    #[error("タグ関係への評価の取り消しに失敗しました")]
    UnrateTagRelationFailed(#[source] anyhow::Error)
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use uuid::Uuid;

    use crate::common::{fallible::Fallible, id::account_id::AccountId, tag::{non_top_tag_id::NonTopTagId, relation::TagRelation, tag_id::TagId}, uuid::uuid4::Uuid4};

    use super::{UnrateTagRelation, UnrateTagRelationError};

    static NON_PROPOSED_RELATION_SUBTAG_ID: LazyLock<NonTopTagId> = LazyLock::new(|| {
        let uuid = Uuid::from_fields(0x01, 0x01, 0x4001, &[0x80, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x03]);
        NonTopTagId::try_from(TagId::of(Uuid4::new_unchecked(uuid))).unwrap()
    });
    struct MockUnrateTagRelation;

    impl UnrateTagRelation for MockUnrateTagRelation {
        async fn is_tag_relation_proposed(&self, _: NonTopTagId, supertag_id: NonTopTagId, _: TagRelation) -> Fallible<bool, UnrateTagRelationError> {
            Ok(supertag_id != *NON_PROPOSED_RELATION_SUBTAG_ID)
        }

        async fn unrate(&self, _: AccountId, _: NonTopTagId, _: NonTopTagId, _: TagRelation) -> Fallible<(), UnrateTagRelationError> {
            Ok(())
        }
    }

    async fn test_rate_tag_relation(subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), UnrateTagRelationError> {
        MockUnrateTagRelation.unrate_tag_relation(AccountId::gen(), subtag_id, supertag_id, relation).await
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
            assert!(matches!(res.err().unwrap(), UnrateTagRelationError::NonProposedTagRelation));
        }
    }
}