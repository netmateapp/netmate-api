use std::{collections::HashSet, sync::Arc};

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, tag::{non_top_tag::NonTopTagId, tag_id::TagId}}, endpoints::tag::proposal::propose::dsl::validate_topology::{ValidateTopology, ValidateTopologyError}};

use super::ProposeTagRelationImpl;

impl ValidateTopology for ProposeTagRelationImpl {
    async fn is_acyclic(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId) -> Fallible<bool, ValidateTopologyError> {
        let maybe_is_unstable_proposal = self.db
            .execute_unpaged(&self.select_subtag, (subtag_id, supertag_id))
            .await
            .map_err(|e| ValidateTopologyError::IsAcyclicFailed(e.into()))?
            .maybe_first_row_typed::<(bool, )>()
            .map_err(|e| ValidateTopologyError::IsAcyclicFailed(e.into()))?;

        // 存在しない又は未安定の提案なら巡回しない
        match maybe_is_unstable_proposal {
            Some((is_unstable_proposal, )) => {
                let is_acyclic = !is_unstable_proposal;
                Ok(is_acyclic)
            },
            None => Ok(false)
        }
    }

    async fn is_equivalent(&self, lesser_tag_id: NonTopTagId, greater_tag_id: NonTopTagId) -> Fallible<bool, ValidateTopologyError> {
        async fn fetch_all_related_tags(db: &Arc<Session>, selector: &Arc<PreparedStatement>, tag_id: NonTopTagId) -> Fallible<HashSet<TagId>, ValidateTopologyError> {
            db.execute_unpaged(selector, (tag_id, ))
            .await
            .map_err(|e| ValidateTopologyError::IsEquivalentFailed(e.into()))?
            .rows_typed()
            .map(|rows| {
                rows.flatten()
                    .filter(|(_, is_unstable_proposal): &(TagId, bool)| !is_unstable_proposal)
                    .map(|(tag_id, _)| tag_id)
                    .collect::<HashSet<TagId>>()
            })
            .map_err(|e| ValidateTopologyError::IsEquivalentFailed(e.into()))
        }

        let mut lesser_tag_all_subtags = fetch_all_related_tags(&self.db, &self.select_all_subtag, lesser_tag_id).await?;
        let mut lesser_tag_all_supertags = fetch_all_related_tags(&self.db, &self.select_all_supertag, lesser_tag_id).await?;
        let mut greater_tag_all_subtags = fetch_all_related_tags(&self.db, &self.select_all_subtag, greater_tag_id).await?;
        let mut greater_tag_all_supertags = fetch_all_related_tags(&self.db, &self.select_all_supertag, greater_tag_id).await?;

        if lesser_tag_all_subtags.len() >= greater_tag_all_subtags.len() {
            std::mem::swap(&mut lesser_tag_all_subtags, &mut greater_tag_all_subtags);
            std::mem::swap(&mut lesser_tag_all_supertags, &mut greater_tag_all_supertags);
        }

        let is_included = lesser_tag_all_subtags.iter().all(|subtag| greater_tag_all_subtags.contains(subtag))
            && lesser_tag_all_supertags.iter().all(|supertag| greater_tag_all_supertags.contains(supertag));

        Ok(is_included)
    }
}