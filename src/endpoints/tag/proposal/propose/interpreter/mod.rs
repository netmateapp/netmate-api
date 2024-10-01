use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::helper::{error::InitError, redis::connection::Pool, scylla::prepare};

pub mod propose;
pub mod validate_topology;

pub struct ProposeTagRelationImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    select_tag_relation_proposal: Arc<PreparedStatement>,
    select_subtag: Arc<PreparedStatement>, // サイクル検出用
    select_all_subtag: Arc<PreparedStatement>, // 同値性の判定用
    select_all_supertag: Arc<PreparedStatement>, // 同上
    select_language_group_and_tag_name: Arc<PreparedStatement>,
    insert_tag_relation_proposal: Arc<PreparedStatement>,
    insert_tag_relation_rating: Arc<PreparedStatement>,
}

impl ProposeTagRelationImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<Self>> {
        let select_tag_relation_proposal = prepare(&db, "SELECT inclusion_or_equivalence FROM tag_relation_proposals WHERE subtag_id = ? AND supertag_id = ?").await?;

        let select_subtag = prepare(&db, "SELECT is_unstable_proposal FROM transitive_closure_and_unstable_proposals WHERE tag_id = ? AND relation = 2 AND related_tag_id = ?").await?;

        let select_all_subtag = prepare(&db, "SELECT related_tag_id, is_unstable_proposal FROM transitive_closure_and_unstable_proposals WHERE tag_id = ? AND relation = 2").await?;
        
        let select_all_supertag = prepare(&db, "SELECT related_tag_id, is_unstable_proposal FROM transitive_closure_and_unstable_proposals WHERE tag_id = ? AND relation = 0").await?;

        let select_language_group_and_tag_name = prepare(&db, "SELECT language_group, name FROM tags WHERE id = ?").await?;

        let insert_tag_relation_proposal = prepare(&db, "INSERT INTO tag_relation_proposals (subtag_id, supertag_id, inclusion_or_equivalence, language_group, proposer_id. proposed_at) VALUES (?, ?, ?, ?, ?, ?) IF NOT EXISTS").await?;

        let insert_tag_relation_rating = prepare(&db, "INSERT INTO tag_relation_ratings_by_account (account_id, subtag_id, supertag_id, inclusion_or_equivalence, operation_id) VALUES (?, ?, ?, ?, 127)").await?;

        Ok(Self {
            db,
            cache,
            select_tag_relation_proposal,
            select_subtag,
            select_all_subtag,
            select_all_supertag,
            select_language_group_and_tag_name,
            insert_tag_relation_proposal,
            insert_tag_relation_rating
        })
    }
}