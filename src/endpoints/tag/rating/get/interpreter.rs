use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, profile::account_id::AccountId, tag::{hierarchy::TagHierarchy, non_top_tag::NonTopTagId, proposal_operation::ProposalOperation, relation::TagRelation}}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{GetTagRelationProposalOperation, GetTagRelationProposalOperationError};

pub struct GetTagRelationRatingImpl {
    db: Arc<Session>,
    select_operation_id: Arc<PreparedStatement>,
    select_is_status_calculated: Arc<PreparedStatement>,
    delete_operation_id_proposed: Arc<PreparedStatement>,
}

impl GetTagRelationRatingImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<Self, InitError<Self>> {
        let select_operation_id = prepare(&db, "SELECT operation_id FROM tag_relation_ratings_by_account WHERE account_id = ? AND subtag_id = ? AND supertag_id = ? AND relation = ?").await?;

        let select_is_status_calculated = prepare(&db, "SELECT is_status_calculated FROM hierarchical_tag_lists WHERE tag_id = ? AND hierarchy = ? AND related_tag_id = ?").await?;

        let delete_operation_id_proposed = prepare(&db, "DELETE FROM tag_relation_ratings_by_account WHERE account_id = ? AND subtag_id = ? AND supertag_id = ? AND relation = ?").await?;

        Ok(Self {
            db,
            select_operation_id,
            select_is_status_calculated,
            delete_operation_id_proposed,
        })
    }
}

impl GetTagRelationProposalOperation for GetTagRelationRatingImpl {
    async fn fetch_tag_relation_proposal_operation(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<Option<ProposalOperation>, GetTagRelationProposalOperationError> {
        self.db
            .execute_unpaged(&self.select_operation_id, (account_id, subtag_id, supertag_id, relation))
            .await
            .map_err(|e| GetTagRelationProposalOperationError::FetchTagRelationRatingOperationFailed(e.into()))?
            .maybe_first_row_typed::<(ProposalOperation, )>()
            .map_err(|e| GetTagRelationProposalOperationError::FetchTagRelationRatingOperationFailed(e.into()))
            .map(|o| o.map(|(operation, )| operation))
    }

    async fn is_proposal_status_uncalculated(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<bool, GetTagRelationProposalOperationError> {
        let hierarchy = match relation {
            TagRelation::Inclusion => TagHierarchy::Super,
            TagRelation::Equivalence => TagHierarchy::Equivalent
        };
        
        self.db
            .execute_unpaged(&self.select_is_status_calculated, (subtag_id, hierarchy, supertag_id))
            .await
            .map_err(|e| GetTagRelationProposalOperationError::IsStatusUncalculatedFailed(e.into()))?
            .first_row_typed::<(bool, )>()
            .map_err(|e| GetTagRelationProposalOperationError::IsStatusUncalculatedFailed(e.into()))
            .map(|(is_status_calculated, )| !is_status_calculated)
    }

    async fn delete_proposal_operation(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), GetTagRelationProposalOperationError> {
        self.db
            .execute_unpaged(&self.delete_operation_id_proposed, (account_id, subtag_id, supertag_id, relation))
            .await
            .map_err(|e| GetTagRelationProposalOperationError::DeflagIsProposerFailed(e.into()))?;

        Ok(())
    }

}