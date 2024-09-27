use thiserror::Error;

use crate::common::{fallible::Fallible, language_group::LanguageGroup, tag::{non_top_tag_id::NonTopTagId, relation::{validate_tag_relation, TagRelation}, top_tag_id::TopTagId}};

pub(crate) trait ProposeTagRelation {
    async fn propose_tag_relation(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), ProposeTagRelationError> {
        match validate_tag_relation(subtag_id, supertag_id, relation) {
            Ok(()) => {
                match relation {
                    TagRelation::Inclusion => if !self.is_acyclic(subtag_id, supertag_id).await? {
                        return Err(ProposeTagRelationError::IsNotAcyclic);
                    },
                    TagRelation::Equivalence => if !self.is_equivalent(subtag_id, supertag_id).await? {
                        return Err(ProposeTagRelationError::IsNotEquivalent);
                    }
                }

                if !self.has_already_been_proposed(subtag_id, supertag_id, relation).await? {
                    let subtag_top_tag = self.fetch_top_tag(subtag_id).await?;
                    let supertag_top_tag = self.fetch_top_tag(supertag_id).await?;
    
                    if subtag_top_tag == supertag_top_tag {
                        let language_group = LanguageGroup::from(subtag_top_tag);
                        self.propose(subtag_id, supertag_id, relation, language_group).await
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

    async fn is_acyclic(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId) -> Fallible<bool, ProposeTagRelationError>;

    async fn is_equivalent(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId) -> Fallible<bool, ProposeTagRelationError>;

    async fn has_already_been_proposed(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<bool, ProposeTagRelationError>;

    async fn fetch_top_tag(&self, tag_id: NonTopTagId) -> Fallible<TopTagId, ProposeTagRelationError>;

    async fn propose(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation, language_group: LanguageGroup) -> Fallible<(), ProposeTagRelationError>;
}

#[derive(Debug, Error)]
pub enum ProposeTagRelationError {
    #[error("非巡回性の判定に失敗しました")]
    IsAcyclicFailed(#[source] anyhow::Error),
    #[error("非巡回ではありません")]
    IsNotAcyclic,
    #[error("同値性の判定に失敗しました")]
    IsEquivalentFailed(#[source] anyhow::Error),
    #[error("同値ではありません")]
    IsNotEquivalent,
    #[error("既に提案されたかどうかの確認に失敗しました")]
    HasAlreadyBeenProposedFailed(#[source] anyhow::Error),
    #[error("既に提案されています")]
    HasAlreadyBeenProposed,
    #[error("トップタグの取得に失敗しました")]
    FetchTopTagFailed(#[source] anyhow::Error),
    #[error("異なる言語グループのタグ間の関係は提案できません")]
    DifferentLanguageGroups,
    #[error("提案に失敗しました")]
    ProposeFailed(#[source] anyhow::Error),
    #[error("タグ関係の提案に失敗しました")]
    ProposeTagRelationFailed(#[source] anyhow::Error),
}