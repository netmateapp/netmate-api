use std::sync::Arc;

use redis::Script;
use scylla::{prepared_statement::PreparedStatement, Session};

use crate::helper::{error::InitError, redis::connection::Pool, scylla::prepare};

pub mod propose;
pub mod update_tag_list;
pub mod validate_topology;

pub struct ProposeTagRelationImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    select_subtag: Arc<PreparedStatement>, // サイクル検出用
    select_all_subtag: Arc<PreparedStatement>, // 同値性の判定用
    select_all_supertag: Arc<PreparedStatement>, // 同値性の判定用
    check_tag_relation_proposal_exists: Arc<PreparedStatement>, // 提案の存在判定
    select_language_group_and_tag_name: Arc<PreparedStatement>, // 言語と名前の取得
    insert_tag_relation_proposal: Arc<PreparedStatement>, // 提案の追加
    insert_tag_relation_rating: Arc<PreparedStatement>, // 提案者フラグを立てる
    insert_unstable_proposals_to_list: Arc<Script>, // 階層別タグ一覧(Redis)に提案を追加
    insert_inclusion_relation_proposal: Arc<PreparedStatement>, // 包含関係の場合に階層別タグ一覧に提案を追加
    insert_equivalence_relation_proposal: Arc<PreparedStatement>, // 同値関係の場合に階層別タグ一覧に提案を追加
}

impl ProposeTagRelationImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<Self>> {
        let select_subtag = prepare(&db, "SELECT is_proposal, is_stable FROM hierarchical_tag_lists WHERE tag_id = ? AND hierarchy = 2 AND related_tag_id = ?").await?;

        let select_all_subtag = prepare(&db, "SELECT related_tag_id, is_proposal, is_stable FROM hierarchical_tag_lists WHERE tag_id = ? AND hierarchy = 2").await?;
        
        let select_all_supertag = prepare(&db, "SELECT related_tag_id, is_proposal, is_stable FROM hierarchical_tag_lists WHERE tag_id = ? AND hierarchy = 0").await?;

        let check_tag_relation_proposal_exists = prepare(&db, "SELECT language_group FROM tag_relation_proposals WHERE subtag_id = ? AND supertag_id = ? AND relation = ?").await?;

        let select_language_group_and_tag_name = prepare(&db, "SELECT language_group, name FROM tags WHERE id = ?").await?;

        let insert_tag_relation_proposal = prepare(&db, "INSERT INTO tag_relation_proposals (subtag_id, supertag_id, relation, language_group, proposer_id, proposed_at) VALUES (?, ?, ?, ?, ?, ?) IF NOT EXISTS").await?;

        let insert_tag_relation_rating = prepare(&db, "INSERT INTO tag_relation_ratings_by_account (account_id, subtag_id, supertag_id, relation, operation_id) VALUES (?, ?, ?, ?, 127)").await?;

        let insert_unstable_proposals_to_list = Arc::new(Script::new(include_str!("add_proposals_to_hierarchical_tag_lists.lua")));

        let insert_inclusion_relation_proposal = prepare(&db, "
            BEGIN BATCH
                INSERT INTO hierarchical_tag_lists (tag_id, hierarchy, related_tag_id, related_tag_name, is_proposal, is_stable, is_status_calculated) VALUES (?, 0, ?, ?, true, false, false);
                INSERT INTO hierarchical_tag_lists (tag_id, hierarchy, related_tag_id, related_tag_name, is_proposal, is_stable, is_status_calculated) VALUES (?, 2, ?, ?, true, false, false);
            APPLY BATCH
        ").await?;

        let insert_equivalence_relation_proposal = prepare(&db, "
            BEGIN BATCH
                INSERT INTO hierarchical_tag_lists (tag_id, hierarchy, related_tag_id, related_tag_name, is_proposal, is_stable, is_status_calculated) VALUES (?, 1, ?, ?, true, false, false);
                INSERT INTO hierarchical_tag_lists (tag_id, hierarchy, related_tag_id, related_tag_name, is_proposal, is_stable, is_status_calculated) VALUES (?, 1, ?, ?, true, false, false);
            APPLY BATCH
        ").await?;

        Ok(Self {
            db,
            cache,
            select_subtag,
            select_all_subtag,
            select_all_supertag,
            check_tag_relation_proposal_exists,
            select_language_group_and_tag_name,
            insert_tag_relation_proposal,
            insert_tag_relation_rating,
            insert_unstable_proposals_to_list,
            insert_inclusion_relation_proposal,
            insert_equivalence_relation_proposal
        })
    }
}