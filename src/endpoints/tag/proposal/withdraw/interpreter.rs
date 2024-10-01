use std::sync::Arc;

use redis::Script;
use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, profile::account_id::AccountId, tag::{hierarchy::TagHierarchy, non_top_tag::NonTopTagId, relation::TagRelation}}, endpoints::tag::PROPOSER_FLAG, helper::{error::InitError, redis::{connection::{conn, Pool}, namespace::NAMESPACE_SEPARATOR, namespaces::{EQUIVALENT, SUB, SUPER, TAG_LIST}}, scylla::prepare}};

use super::dsl::{WithdrawTagRelationProposal, WithdrawTagRelationProposalError};

pub struct WithdrawTagRelationProposalImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    select_operation_id: Arc<PreparedStatement>,
    select_is_status_calculated: Arc<PreparedStatement>,
    delete_operation_id_proposed: Arc<PreparedStatement>,
    delete_proposal: Arc<PreparedStatement>,
    delete_from_hierarchical_tag_list: Arc<PreparedStatement>,
    remove_proposals_from_hierarchical_tag_lists: Arc<Script>
}

impl WithdrawTagRelationProposalImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<Self>> {
        let select_operation_id = prepare(&db, "SELECT operation_id FROM tag_relation_ratings_by_account WHERE account_id = ? AND subtag_id = ? AND supertag_id = ? AND relation = ?").await?;

        let select_is_status_calculated = prepare(&db, "SELECT is_status_calculated FROM hierarchical_tag_lists WHERE tag_id = ? AND hierarchy = ? AND related_tag_id = ?").await?;

        let delete_operation_id_proposed = prepare(&db, "DELETE FROM tag_relation_ratings_by_account WHERE account_id = ? AND subtag_id = ? AND supertag_id = ? AND relation = ?").await?;

        let delete_proposal = prepare(&db, "DELETE FROM tag_relation_proposals WHERE account_id = ? AND subtag_id = ? AND supertag_id = ? AND relation = ?").await?;

        let delete_from_hierarchical_tag_list = prepare(&db, "DELETE FROM hierarchical_tag_lists WHERE tag_id = ? AND hierarchy = ? AND related_tag_id = ?").await?;

        let remove_proposals_from_hierarchical_tag_lists = Arc::new(Script::new(include_str!("remove_proposals_from_hierarchical_tag_lists.lua")));

        Ok(Self {
            db,
            cache,
            select_operation_id,
            select_is_status_calculated,
            delete_operation_id_proposed,
            delete_proposal,
            delete_from_hierarchical_tag_list,
            remove_proposals_from_hierarchical_tag_lists
        })
    }
}

impl WithdrawTagRelationProposal for WithdrawTagRelationProposalImpl {
    async fn is_proposer(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<bool, WithdrawTagRelationProposalError> {
        self.db
            .execute_unpaged(&self.select_operation_id, (account_id, subtag_id, supertag_id, relation))
            .await
            .map_err(|e| WithdrawTagRelationProposalError::IsProposerFailed(e.into()))?
            .first_row_typed::<(i8, )>()
            .map_err(|e| WithdrawTagRelationProposalError::IsProposerFailed(e.into()))
            .map(|(operation_id, )| operation_id == PROPOSER_FLAG)
    }

    async fn is_status_calculated(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, hierarchy: TagHierarchy) -> Fallible<bool, WithdrawTagRelationProposalError> {
        self.db
            .execute_unpaged(&self.select_is_status_calculated, (subtag_id, hierarchy, supertag_id))
            .await
            .map_err(|e| WithdrawTagRelationProposalError::IsStatusCalculatedFailed(e.into()))?
            .first_row_typed::<(bool, )>()
            .map_err(|e| WithdrawTagRelationProposalError::IsStatusCalculatedFailed(e.into()))
            .map(|(is_status_calculated, )| is_status_calculated)
    }

    async fn deflag_is_proposer(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), WithdrawTagRelationProposalError> {
        self.db
            .execute_unpaged(&self.delete_operation_id_proposed, (account_id, subtag_id, supertag_id, relation))
            .await
            .map_err(|e| WithdrawTagRelationProposalError::DeflagIsProposerFailed(e.into()))?;

        Ok(())
    }

    async fn withdraw(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), WithdrawTagRelationProposalError> {
        // 提案を削除
        self.db
            .execute_unpaged(&self.delete_proposal, (account_id, subtag_id, supertag_id, relation))
            .await
            .map_err(|e| WithdrawTagRelationProposalError::WithdrawFailed(e.into()))?;

        async fn delete_from_hierarchical_tag_list(db: &Arc<Session>, delete_from_hierarchical_tag_list: &Arc<PreparedStatement>, tag_id: NonTopTagId, hierarchy: TagHierarchy, related_tag_id: NonTopTagId) -> Fallible<(), WithdrawTagRelationProposalError> {
            db
                .execute_unpaged(delete_from_hierarchical_tag_list, (tag_id, hierarchy, related_tag_id))
                .await
                .map_err(|e| WithdrawTagRelationProposalError::WithdrawFailed(e.into()))?;
            Ok(())
        }

        let mut conn = conn(&self.cache, |e| WithdrawTagRelationProposalError::WithdrawFailed(e.into())).await?;

        // 階層別タグ一覧から除去
        match relation {
            TagRelation::Inclusion => {
                delete_from_hierarchical_tag_list(&self.db, &self.delete_from_hierarchical_tag_list, subtag_id, TagHierarchy::Super, supertag_id).await?;
                delete_from_hierarchical_tag_list(&self.db, &self.delete_from_hierarchical_tag_list, supertag_id, TagHierarchy::Sub, subtag_id).await?;

                self.remove_proposals_from_hierarchical_tag_lists
                    .key(format!("{}{}{}{}{}", TAG_LIST, NAMESPACE_SEPARATOR, subtag_id, NAMESPACE_SEPARATOR, SUPER))
                    .arg(supertag_id)
                    .key(format!("{}{}{}{}{}", TAG_LIST, NAMESPACE_SEPARATOR, supertag_id, NAMESPACE_SEPARATOR, SUB))
                    .arg(subtag_id)
                    .invoke_async(&mut *conn)
                    .await
                    .map_err(|e| WithdrawTagRelationProposalError::WithdrawFailed(e.into()))?;
            },
            TagRelation::Equivalence => {
                let lesser_tag_id = subtag_id;
                let greater_tag_id = supertag_id;

                delete_from_hierarchical_tag_list(&self.db, &self.delete_from_hierarchical_tag_list, lesser_tag_id, TagHierarchy::Equivalent, greater_tag_id).await?;
                delete_from_hierarchical_tag_list(&self.db, &self.delete_from_hierarchical_tag_list, greater_tag_id, TagHierarchy::Equivalent, lesser_tag_id).await?;

                self.remove_proposals_from_hierarchical_tag_lists
                    .key(format!("{}{}{}{}{}", TAG_LIST, NAMESPACE_SEPARATOR, lesser_tag_id, NAMESPACE_SEPARATOR, EQUIVALENT))
                    .arg(lesser_tag_id)
                    .key(format!("{}{}{}{}{}", TAG_LIST, NAMESPACE_SEPARATOR, greater_tag_id, NAMESPACE_SEPARATOR, EQUIVALENT))
                    .arg(greater_tag_id)
                    .invoke_async(&mut *conn)
                    .await
                    .map_err(|e| WithdrawTagRelationProposalError::WithdrawFailed(e.into()))?;
            },
        }

        Ok(())
    }
}