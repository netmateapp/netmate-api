use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::helper::{error::InitError, redis::connection::Pool, scylla::prepare};

pub mod propose;
pub mod validate_topology;

pub struct ProposeTagRelationImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    select_tag_relation: Arc<PreparedStatement>,
    select_top_tag: Arc<PreparedStatement>,
    insert_tag_relation: Arc<PreparedStatement>,
    insert_tag_relation_rating: Arc<PreparedStatement>,
}

impl ProposeTagRelationImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<Self>> {
        let select_tag_relation = prepare(&db, "SELECT inclusion_or_equivalence FROM proposed_tag_relations WHERE subtag_id = ? AND supertag_id = ?").await?;

        let select_top_tag = prepare(&db, "SELECT supertag_id FROM proposed_tag_relations WHERE subtag_id = ? LIMIT 1").await?;

        let insert_tag_relation = prepare(&db, "INSERT INTO proposed_tag_relations (subtag_id, supertag_id, inclusion_or_equivalence, language_group) VALUES (?, ?, ?, ?) IF NOT EXISTS").await?;

        let insert_tag_relation_rating = prepare(&db, "INSERT INTO tag_relation_ratings_by_account (account_id, subtag_id, supertag_id, inclusion_or_equivalence, operation_id) VALUES (?, ?, ?, ?, 127)").await?;

        Ok(Self { db, cache, select_tag_relation, select_top_tag, insert_tag_relation, insert_tag_relation_rating })
    }
}