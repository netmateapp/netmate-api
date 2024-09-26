use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{cycle::Cycle, fallible::Fallible, id::account_id::AccountId, rating::Rating, tag::{relation::TagRelation, tag_id::TagId}}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{RateTagRelation, RateTagRelationError};

pub struct RateTagRelationImpl {
    db: Arc<Session>,
    select_inclusion_or_equivalence: Arc<PreparedStatement>,
    insert_tag_relation_rating_to_account: Arc<PreparedStatement>,
    insert_tag_relation_rating_to_cycle: Arc<PreparedStatement>
}

impl RateTagRelationImpl {
    pub async fn try_new(db: Arc<Session>) -> Fallible<Self, InitError<Self>> {
        let select_inclusion_or_equivalence = prepare(&db, "SELECT inclusion_or_equivalence FROM tag_relations WHERE subtag_id = ? AND supertag_id = ?").await?;

        let insert_tag_relation_rating = prepare(&db, "INSERT INTO tag_relation_rating_operations_by_account (account_id, subtag_id, supertag_id, inclusion_or_equivalence, operation_id) VALUES (?, ?, ?, ?, ?)").await?;

        let insert_tag_relation_rating_to_cycle = prepare(&db, "INSERT INTO tag_relation_rating_operations_by_cycle (cycle, account_id, subtag_id, supertag_id, inclusion_or_equivalence, operation_id) VALUES (?, ?, ?, ?, ?, ?)").await?;

        Ok(Self{ db, select_inclusion_or_equivalence, insert_tag_relation_rating_to_account: insert_tag_relation_rating, insert_tag_relation_rating_to_cycle })
    }
}

impl RateTagRelation for RateTagRelationImpl {
    // 提案は撤回される可能性があるため、キャッシュできない
    async fn is_tag_relation_suggested(&self, subtag_id: TagId, supertag_id: TagId, relation: TagRelation) -> Fallible<bool, RateTagRelationError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> RateTagRelationError {
            RateTagRelationError::CheckSuggestedTagRelationFailed(e.into())
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

    async fn rate(&self, account_id: AccountId, subtag_id: TagId, supertag_id: TagId, relation: TagRelation, rating: Rating) -> Fallible<(), RateTagRelationError> {
        self.db
            .execute_unpaged(&self.insert_tag_relation_rating_to_account, (account_id, subtag_id, supertag_id, relation, rating))
            .await
            .map_err(|e| RateTagRelationError::RateTagRelationFailed(e.into()))?;

        self.db
            .execute_unpaged(&self.insert_tag_relation_rating_to_cycle, (Cycle::current_cycle(), account_id, subtag_id, supertag_id, relation, rating))
            .await
            .map_err(|e| RateTagRelationError::RateTagRelationFailed(e.into()))?;

        Ok(())
    }
}