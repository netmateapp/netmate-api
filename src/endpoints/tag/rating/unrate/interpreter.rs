use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::fallible::Fallible, helper::{error::InitError, scylla::prepare}};

pub struct UnrateTagRelationImpl {
    db: Arc<Session>,
    select_inclusion_or_equivalence: Arc<PreparedStatement>,
    delete_tag_relation_rating: Arc<PreparedStatement>,
    delete_tag_relation_rating_log: Arc<PreparedStatement>
}

impl UnrateTagRelationImpl {
    pub async fn try_new(db: Arc<Session>) -> Fallible<Self, InitError<Self>> {
        let select_inclusion_or_equivalence = prepare(&db, "SELECT inclusion_or_equivalence FROM tag_relations WHERE subtag_id = ? AND supertag_id = ?").await?;

        let delete_tag_relation_rating = prepare(&db, "INSERT INTO tag_relation_rating_operations_by_cycle (account_id, subtag_id, supertag_id, inclusion_or_equivalence, operation_id) VALUES (?, ?, ?, ?, ?)").await?;

        let delete_tag_relation_rating_log = prepare(&db, "INSERT INTO tag_relation_rating_operations_by_account (cycle, account_id, subtag_id, supertag_id, inclusion_or_equivalence, operation_id) VALUES (?, ?, ?, ?, ?, ?)").await?;

        Ok(Self{ db, select_inclusion_or_equivalence, delete_tag_relation_rating, delete_tag_relation_rating_log })
    }
}
