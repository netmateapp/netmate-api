use crate::{common::{fallible::Fallible, id::account_id::AccountId, language_group::LanguageGroup, tag::{non_top_tag_id::NonTopTagId, relation::TagRelation, top_tag_id::TopTagId}}, endpoints::tag::proposal::propose::dsl::propose::{ProposeTagRelation, ProposeTagRelationError}};

use super::ProposeTagRelationImpl;

impl ProposeTagRelation for ProposeTagRelationImpl {
    async fn has_already_been_proposed(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Fallible<bool, ProposeTagRelationError> {
        self.db
            .execute_unpaged(&self.select_tag_relation, (subtag_id, supertag_id))
            .await
            .map_err(|e| ProposeTagRelationError::HasAlreadyBeenProposedFailed(e.into()))?
            .maybe_first_row_typed::<(TagRelation, )>()
            .map_err(|e| ProposeTagRelationError::HasAlreadyBeenProposedFailed(e.into()))
            .map(|o| match o {
                Some((inclusion_or_equivalence, )) => relation == inclusion_or_equivalence,
                None => false,
            })
    }

    async fn fetch_top_tag(&self, tag_id: NonTopTagId) -> Fallible<Option<TopTagId>, ProposeTagRelationError> {
        self.db
            .execute_unpaged(&self.select_top_tag, (tag_id, ))
            .await
            .map_err(|e| ProposeTagRelationError::FetchTopTagFailed(e.into()))?
            .maybe_first_row_typed::<(TopTagId, )>()
            .map_err(|e| ProposeTagRelationError::FetchTopTagFailed(e.into()))
            .map(|o| o.map(|(top_tag_id, )| top_tag_id))
    }

    async fn propose(&self, account_id: AccountId, subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation, language_group: LanguageGroup) -> Fallible<(), ProposeTagRelationError> {
        self.db
            .execute_unpaged(&self.insert_tag_relation, (subtag_id, supertag_id, relation, language_group))
            .await
            .map_err(|e| ProposeTagRelationError::ProposeFailed(e.into()))?;

        self.db
            .execute_unpaged(&self.insert_tag_relation_rating, (account_id, subtag_id, supertag_id, relation))
            .await
            .map_err(|e| ProposeTagRelationError::ProposeFailed(e.into()))?;

        Ok(())
    }
}
