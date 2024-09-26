use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{cycle::Cycle, fallible::Fallible, id::account_id::AccountId, tag::{relation::TagRelation, tag_id::TagId}}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{UnrateTagRelation, UnrateTagRelationError};

pub struct UnrateTagRelationImpl {
    db: Arc<Session>,
    select_inclusion_or_equivalence: Arc<PreparedStatement>,
    delete_tag_relation_rating_from_account: Arc<PreparedStatement>,
    insert_tag_relation_rating_removal_into_cycle: Arc<PreparedStatement>
}

impl UnrateTagRelationImpl {
    pub async fn try_new(db: Arc<Session>) -> Fallible<Self, InitError<Self>> {
        let select_inclusion_or_equivalence = prepare(&db, "SELECT inclusion_or_equivalence FROM tag_relations WHERE subtag_id = ? AND supertag_id = ?").await?;

        let delete_tag_relation_rating_from_account = prepare(&db, "DELETE FROM tag_relation_rating_operations_by_account WHERE account_id = ? AND subtag_id = ? AND supertag_id = ? AND inclusion_or_equivalence = ?").await?;

        let insert_tag_relation_rating_removal_into_cycle = prepare(&db, "INSERT INTO tag_relation_rating_operations_by_cycle (cycle, account_id, subtag_id, supertag_id, inclusion_or_equivalence, operation_id) VALUES (?, ?, ?, ?, ?, 255)").await?;

        Ok(Self{ db, select_inclusion_or_equivalence, delete_tag_relation_rating_from_account, insert_tag_relation_rating_removal_into_cycle })
    }
}

impl UnrateTagRelation for UnrateTagRelationImpl {
    async fn is_tag_relation_suggested(&self, subtag_id: TagId, supertag_id: TagId, relation: TagRelation) -> Fallible<bool, UnrateTagRelationError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> UnrateTagRelationError {
            UnrateTagRelationError::CheckSuggestedTagRelationFailed(e.into())
        }
        
        self.db
            .execute_unpaged(&self.select_inclusion_or_equivalence, (subtag_id, supertag_id))
            .await
            .map_err(handle_error)?
            .maybe_first_row_typed::<(TagRelation, )>()
            .map_err(handle_error)
            .map(|o| match o {
                Some((inclusion_or_equivalence, )) => inclusion_or_equivalence == relation,
                None => false
            })
    }

    async fn unrate(&self, account_id: AccountId, subtag_id: TagId, supertag_id: TagId, relation: TagRelation) -> Fallible<(), UnrateTagRelationError> {
        self.db
        .execute_unpaged(&self.delete_tag_relation_rating_from_account, (account_id, subtag_id, supertag_id, relation))
        .await
        .map_err(|e| UnrateTagRelationError::UnrateTagRelationFailed(e.into()))?;

        self.db
            .execute_unpaged(&self.insert_tag_relation_rating_removal_into_cycle, (Cycle::current_cycle(), account_id, subtag_id, supertag_id, relation))
            .await
            .map_err(|e| UnrateTagRelationError::UnrateTagRelationFailed(e.into()))?;

        Ok(())
    }
}