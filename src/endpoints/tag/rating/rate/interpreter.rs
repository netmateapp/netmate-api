use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{cycle::Cycle, fallible::Fallible, profile::account_id::AccountId, rating::Rating, tag::{language_group::LanguageGroup, non_top_tag::NonTopTagId, relation::TagRelation}}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{RateTagRelation, RateTagRelationError};

pub struct RateTagRelationImpl {
    db: Arc<Session>,
    select_inclusion_or_equivalence: Arc<PreparedStatement>,
    insert_tag_relation_rating_to_account: Arc<PreparedStatement>,
    insert_tag_relation_rating_to_cycle: Arc<PreparedStatement>
}

impl RateTagRelationImpl {
    pub async fn try_new(db: Arc<Session>) -> Fallible<Self, InitError<Self>> {
        let select_inclusion_or_equivalence = prepare(&db, "SELECT language_group FROM tag_relation_proposals WHERE subtag_id = ? AND supertag_id = ? AND relation = ?").await?;

        let insert_tag_relation_rating = prepare(&db, "INSERT INTO tag_relation_ratings_by_account (account_id, subtag_id, supertag_id, relation, operation_id) VALUES (?, ?, ?, ?, ?)").await?;

        let insert_tag_relation_rating_to_cycle = prepare(&db, "INSERT INTO tag_relation_ratings (language_group, cycle, account_id, subtag_id, supertag_id, relation, operation_id) VALUES (?, ?, ?, ?, ?, ?, ?)").await?;

        Ok(Self{ db, select_inclusion_or_equivalence, insert_tag_relation_rating_to_account: insert_tag_relation_rating, insert_tag_relation_rating_to_cycle })
    }
}

impl RateTagRelation for RateTagRelationImpl {
    // 提案は撤回される可能性があるため、キャッシュできない
    async fn fetch_tag_relation_proposed(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<Option<LanguageGroup>, RateTagRelationError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> RateTagRelationError {
            RateTagRelationError::CheckProposedTagRelationFailed(e.into())
        }
        
        self.db
            .execute_unpaged(&self.select_inclusion_or_equivalence, (subtag_id, supertag_id, relation))
            .await
            .map_err(handle_error)?
            .maybe_first_row_typed::<(LanguageGroup, )>()
            .map_err(handle_error)
            .map(|o| o.map(|(language_group, )| language_group))
    }

    async fn rate(&self, language_group: LanguageGroup, cycle: Cycle, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation, rating: Rating) -> Fallible<(), RateTagRelationError> {
        self.db
            .execute_unpaged(&self.insert_tag_relation_rating_to_account, (account_id, subtag_id, supertag_id, relation, rating))
            .await
            .map_err(|e| RateTagRelationError::RateTagRelationFailed(e.into()))?;

        self.db
            .execute_unpaged(&self.insert_tag_relation_rating_to_cycle, (language_group, cycle, account_id, subtag_id, supertag_id, relation, rating))
            .await
            .map_err(|e| RateTagRelationError::RateTagRelationFailed(e.into()))?;

        Ok(())
    }
}