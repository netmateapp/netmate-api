use thiserror::Error;

use crate::common::{fallible::Fallible, profile::account_id::AccountId, tag::{language_group::LanguageGroup, non_top_tag::NonTopTagId, relation::{validate_tag_relation, TagRelation}, tag_name::TagName}};

use super::{add_relation::{HierarchicalTagRelator, HierarchicalTagRelatorError}, validate_topology::{ValidateTopology, ValidateTopologyError}};

pub(crate) trait ProposeTagRelation {
    // 引数に渡されるIDのタグは、存在することが保証されていない
    async fn propose_tag_relation(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), ProposeTagRelationError>
    where
        Self: ValidateTopology + HierarchicalTagRelator
    {
        match validate_tag_relation(subtag_id, supertag_id, relation) {
            Ok(()) => {
                self.validate_topology(subtag_id, supertag_id, relation)
                    .await
                    .map_err(ProposeTagRelationError::InvalidTopology)?;

                if self.has_already_been_proposed(subtag_id, supertag_id, relation).await? {
                    Err(ProposeTagRelationError::HasAlreadyBeenProposed)
                } else {
                    let (subtag_language_group, subtag_name) = self.get_language_group_and_tag_name(subtag_id).await?;
                    let (supertag_language_group, supertag_name) = self.get_language_group_and_tag_name(supertag_id).await?;
    
                    if subtag_language_group == supertag_language_group {
                        self.propose(account_id, subtag_id, supertag_id, relation, subtag_language_group).await?;

                        self.relate_hierarchical_tags(subtag_id, subtag_name, supertag_id, supertag_name, relation)
                            .await
                            .map_err(ProposeTagRelationError::UpdateTagRelationListFailed)
                    } else {
                        Err(ProposeTagRelationError::DifferentLanguageGroups)
                    }
                }
            },
            Err(e) => Err(ProposeTagRelationError::ProposeTagRelationFailed(e.into()))
        }
    }

    async fn has_already_been_proposed(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<bool, ProposeTagRelationError>;

    async fn get_language_group_and_tag_name(&self, tag_id: NonTopTagId) -> Fallible<(LanguageGroup, TagName), ProposeTagRelationError> {
        self.fetch_language_group_and_tag_name(tag_id)
            .await?
            .ok_or_else(|| ProposeTagRelationError::NonExistentTag)
    }

    // タグが存在しない可能性もあるのでOption
    async fn fetch_language_group_and_tag_name(&self, tag_id: NonTopTagId) -> Fallible<Option<(LanguageGroup, TagName)>, ProposeTagRelationError>;

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
    #[error("タグ一覧の更新に失敗しました")]
    UpdateTagRelationListFailed(#[source] HierarchicalTagRelatorError),
    #[error("タグ関係の提案に失敗しました")]
    ProposeTagRelationFailed(#[source] anyhow::Error),
}