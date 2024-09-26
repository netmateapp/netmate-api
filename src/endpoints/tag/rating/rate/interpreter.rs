use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, id::account_id::AccountId, tag::tag_id::TagId}, endpoints::tag::rating::value::{AccountTableOperationId, CycleTableOperationId, Rating, TagRelationType}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{RateTagRelation, RateTagRelationError};

pub struct RateTagRelationImpl {
    db: Arc<Session>,
    insert_tag_relation_rating: Arc<PreparedStatement>,
    insert_tag_relation_rating_log: Arc<PreparedStatement>
}

impl RateTagRelationImpl {
    pub async fn try_new(db: Arc<Session>) -> Fallible<Self, InitError<Self>> {
        let insert_tag_relation_rating = prepare(&db, "INSERT INTO tag_relation_rating_operations_by_cycle (account_id, subtag_id, supertag_id, inclusion_or_equivalence, operation_id) VALUES (?, ?, ?, ?, ?)").await?;
        let insert_tag_relation_rating_log = prepare(&db, "INSERT INTO tag_relation_rating_operations_by_account (cycle, account_id, subtag_id, supertag_id, inclusion_or_equivalence, operation_id) VALUES (?, ?, ?, ?, ?, ?)").await?;
        Ok(Self{ db, insert_tag_relation_rating, insert_tag_relation_rating_log })
    }
}

impl RateTagRelation for RateTagRelationImpl {
    async fn rate(&self, account_id: AccountId, subtag_id: TagId, supertag_id: TagId, inclusion_or_equivalence: TagRelationType, rating: Rating) -> Fallible<(), RateTagRelationError> {
        self.db
            .execute_unpaged(&self.insert_tag_relation_rating, (account_id, subtag_id, supertag_id, inclusion_or_equivalence, AccountTableOperationId::from(rating)))
            .await
            .map_err(|e| RateTagRelationError::RateTagRelationFailed(e.into()))?;

        self.db
            .execute_unpaged(&self.insert_tag_relation_rating_log, (account_id, subtag_id, supertag_id, inclusion_or_equivalence, CycleTableOperationId::from(rating)))
            .await
            .map_err(|e| RateTagRelationError::RateTagRelationFailed(e.into()))?;

        Ok(())
    }
}