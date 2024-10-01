use crate::{common::{fallible::Fallible, profile::account_id::AccountId, tag::{language_group::LanguageGroup, non_top_tag::NonTopTagId, relation::TagRelation, tag_name::TagName}, unixtime::UnixtimeMillis}, endpoints::tag::proposal::propose::dsl::propose::{ProposeTagRelation, ProposeTagRelationError}, helper::scylla::Transactional};

use super::ProposeTagRelationImpl;

impl ProposeTagRelation for ProposeTagRelationImpl {
    async fn has_already_been_proposed(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<bool, ProposeTagRelationError> {
        self.db
            .execute_unpaged(&self.select_tag_relation_proposal, (subtag_id, supertag_id, relation))
            .await
            .map_err(|e| ProposeTagRelationError::HasAlreadyBeenProposedFailed(e.into()))?
            .maybe_first_row_typed::<(LanguageGroup, )>()
            .map_err(|e| ProposeTagRelationError::HasAlreadyBeenProposedFailed(e.into()))
            .map(|o| o.is_some())
    }

    async fn fetch_language_group_and_tag_name(&self, tag_id: NonTopTagId) -> Fallible<Option<(LanguageGroup, TagName)>, ProposeTagRelationError> {
        self.db
            .execute_unpaged(&self.select_language_group_and_tag_name, (tag_id, ))
            .await
            .map_err(|e| ProposeTagRelationError::FetchTopTagFailed(e.into()))?
            .maybe_first_row_typed::<(LanguageGroup, TagName)>()
            .map_err(|e| ProposeTagRelationError::FetchTopTagFailed(e.into()))
    }

    async fn propose(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation, language_group: LanguageGroup) -> Fallible<(), ProposeTagRelationError> {
        self.db
            .execute_unpaged(&self.insert_tag_relation_proposal, (subtag_id, supertag_id, relation, language_group, account_id, UnixtimeMillis::now()))
            .await
            .applied(ProposeTagRelationError::ProposeFailed, || ProposeTagRelationError::HasAlreadyBeenProposed)?;

        self.db
            .execute_unpaged(&self.insert_tag_relation_rating, (account_id, subtag_id, supertag_id, relation))
            .await
            .map_err(|e| ProposeTagRelationError::ProposeFailed(e.into()))?;

        Ok(())
    }
}
