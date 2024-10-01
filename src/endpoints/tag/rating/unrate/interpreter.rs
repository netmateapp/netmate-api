use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{cycle::Cycle, fallible::Fallible, profile::account_id::AccountId, tag::{language_group::LanguageGroup, non_top_tag::NonTopTagId, relation::TagRelation}}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{UnrateTagRelation, UnrateTagRelationError};

pub struct UnrateTagRelationImpl {
    db: Arc<Session>,
    select_inclusion_or_equivalence: Arc<PreparedStatement>,
    delete_tag_relation_rating_from_account: Arc<PreparedStatement>,
    insert_tag_relation_rating_removal_into_cycle: Arc<PreparedStatement>
}

impl UnrateTagRelationImpl {
    pub async fn try_new(db: Arc<Session>) -> Fallible<Self, InitError<Self>> {
        let select_inclusion_or_equivalence = prepare(&db, "SELECT language_group FROM tag_relation_proposals WHERE subtag_id = ? AND supertag_id = ? AND inclusion_or_equivalence = ?").await?;

        let delete_tag_relation_rating_from_account = prepare(&db, "DELETE FROM tag_relation_ratings_by_account WHERE account_id = ? AND subtag_id = ? AND supertag_id = ? AND inclusion_or_equivalence = ?").await?;

        let insert_tag_relation_rating_removal_into_cycle = prepare(&db, "INSERT INTO tag_relation_ratings_by_language_group_and_cycle (language_group, cycle, account_id, subtag_id, supertag_id, inclusion_or_equivalence, operation_id) VALUES (?, ?, ?, ?, ?, ?, 127)").await?;

        Ok(Self{ db, select_inclusion_or_equivalence, delete_tag_relation_rating_from_account, insert_tag_relation_rating_removal_into_cycle })
    }
}

impl UnrateTagRelation for UnrateTagRelationImpl {
    async fn fetch_tag_relation_proposed(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<Option<LanguageGroup>, UnrateTagRelationError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> UnrateTagRelationError {
            UnrateTagRelationError::CheckProposedTagRelationFailed(e.into())
        }
        
        self.db
            .execute_unpaged(&self.select_inclusion_or_equivalence, (subtag_id, supertag_id, relation))
            .await
            .map_err(handle_error)?
            .maybe_first_row_typed::<(LanguageGroup, )>()
            .map_err(handle_error)
            .map(|o| o.map(|(language_group, )| language_group))
    }

    async fn unrate(&self, language_group: LanguageGroup, cycle: Cycle, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<(), UnrateTagRelationError> {
        self.db
        .execute_unpaged(&self.delete_tag_relation_rating_from_account, (account_id, subtag_id, supertag_id, relation))
        .await
        .map_err(|e| UnrateTagRelationError::UnrateTagRelationFailed(e.into()))?;

        self.db
            .execute_unpaged(&self.insert_tag_relation_rating_removal_into_cycle, (language_group, cycle, account_id, subtag_id, supertag_id, relation))
            .await
            .map_err(|e| UnrateTagRelationError::UnrateTagRelationFailed(e.into()))?;

        Ok(())
    }
}