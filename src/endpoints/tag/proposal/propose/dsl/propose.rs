use thiserror::Error;

use crate::common::{fallible::Fallible, profile::account_id::AccountId, tag::{language_group::LanguageGroup, non_top_tag_id::NonTopTagId, relation::{validate_tag_relation, TagRelation}, top_tag_id::TopTagId}};

use super::validate_topology::{ValidateTopology, ValidateTopologyError};

pub(crate) trait ProposeTagRelation {
    async fn propose_tag_relation(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), ProposeTagRelationError>
    where
        Self: ValidateTopology
    {
        match validate_tag_relation(subtag_id, supertag_id, relation) {
            Ok(()) => {
                self.validate_topology(subtag_id, supertag_id, relation)
                    .await
                    .map_err(ProposeTagRelationError::InvalidTopology)?;

                if !self.has_already_been_proposed(subtag_id, supertag_id, relation).await? {
                    let subtag_top_tag = self.top_tag_of(subtag_id).await?;
                    let supertag_top_tag = self.top_tag_of(supertag_id).await?;
    
                    if subtag_top_tag == supertag_top_tag {
                        let language_group = LanguageGroup::from(subtag_top_tag);
                        self.propose(account_id, subtag_id, supertag_id, relation, language_group).await
                    } else {
                        Err(ProposeTagRelationError::DifferentLanguageGroups)
                    }
                } else {
                    Err(ProposeTagRelationError::HasAlreadyBeenProposed)
                }
            },
            Err(e) => Err(ProposeTagRelationError::ProposeTagRelationFailed(e.into()))
        }
    }

    async fn has_already_been_proposed(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<bool, ProposeTagRelationError>;

    async fn top_tag_of(&self, tag_id: NonTopTagId) -> Fallible<TopTagId, ProposeTagRelationError> {
        self.fetch_top_tag(tag_id)
            .await?
            .ok_or_else(|| ProposeTagRelationError::NonExistentTag)
    }

    async fn fetch_top_tag(&self, tag_id: NonTopTagId) -> Fallible<Option<TopTagId>, ProposeTagRelationError>;

    async fn propose(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation, language_group: LanguageGroup) -> Fallible<(), ProposeTagRelationError>;
}

#[derive(Debug, Error)]
pub enum ProposeTagRelationError {
    #[error("無効なトポロジーです")]
    InvalidTopology(#[source] ValidateTopologyError),
    #[error("既に提案されたかどうかの確認に失敗しました")]
    HasAlreadyBeenProposedFailed(#[source] anyhow::Error),
    #[error("既に提案されています")]
    HasAlreadyBeenProposed,
    #[error("トップタグの取得に失敗しました")]
    FetchTopTagFailed(#[source] anyhow::Error),
    #[error("存在しないタグです")]
    NonExistentTag,
    #[error("異なる言語グループのタグ間の関係は提案できません")]
    DifferentLanguageGroups,
    #[error("提案に失敗しました")]
    ProposeFailed(#[source] anyhow::Error),
    #[error("タグ関係の提案に失敗しました")]
    ProposeTagRelationFailed(#[source] anyhow::Error),
}