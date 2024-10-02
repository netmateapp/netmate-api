use std::{collections::HashSet, sync::Arc};

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{consensus::{is_unstable_proposal, proposal::ItemType, stability::Stability}, fallible::Fallible, tag::{non_top_tag::NonTopTagId, tag_id::TagId}}, endpoints::tag::proposal::propose::dsl::validate_topology::{ValidateTopology, ValidateTopologyError}};

use super::ProposeTagRelationImpl;

impl ValidateTopology for ProposeTagRelationImpl {
    async fn is_acyclic(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId) -> Fallible<bool, ValidateTopologyError> {
        let maybe_is_unstable_proposal = self.db
            .execute_unpaged(&self.select_subtag, (subtag_id, supertag_id))
            .await
            .map_err(|e| ValidateTopologyError::IsAcyclicFailed(e.into()))?
            .maybe_first_row_typed::<(ItemType, Stability)>()
            .map_err(|e| ValidateTopologyError::IsAcyclicFailed(e.into()))?;

        // 存在しない又は未安定の提案なら巡回しない
        match maybe_is_unstable_proposal {
            Some((is_proposal, is_stable)) => Ok(is_unstable_proposal(is_proposal, is_stable)),
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
                    .filter(|(_, is_proposal, is_stable): &(TagId, bool, bool)| !(*is_proposal && !*is_stable))
                    .map(|(tag_id, _, _)| tag_id)
                    .collect::<HashSet<TagId>>()
            })
            .map_err(|e| ValidateTopologyError::IsEquivalentFailed(e.into()))
        }

        let lesser_tag_all_subtags = fetch_all_related_tags(&self.db, &self.select_all_subtag, lesser_tag_id).await?;
        let lesser_tag_all_supertags = fetch_all_related_tags(&self.db, &self.select_all_supertag, lesser_tag_id).await?;
        let greater_tag_all_subtags = fetch_all_related_tags(&self.db, &self.select_all_subtag, greater_tag_id).await?;
        let greater_tag_all_supertags = fetch_all_related_tags(&self.db, &self.select_all_supertag, greater_tag_id).await?;

        Ok(can_relate_by_equivalence(lesser_tag_all_subtags, lesser_tag_all_supertags, greater_tag_all_subtags, greater_tag_all_supertags))
    }
}

fn can_relate_by_inclusion(related_tag_status: Option<(ItemType, Stability)>) -> bool {
    match related_tag_status {
        Some((is_proposal, is_stable)) => is_unstable_proposal(is_proposal, is_stable),
        None => false
    }
}

fn can_relate_by_equivalence(mut lesser_tag_all_subtags: HashSet<TagId>, mut lesser_tag_all_supertags: HashSet<TagId>, mut greater_tag_all_subtags: HashSet<TagId>, mut greater_tag_all_supertags: HashSet<TagId>) -> bool {
    if lesser_tag_all_subtags.len() >= greater_tag_all_subtags.len() {
        std::mem::swap(&mut lesser_tag_all_subtags, &mut greater_tag_all_subtags);
        std::mem::swap(&mut lesser_tag_all_supertags, &mut greater_tag_all_supertags);
    }
    
    lesser_tag_all_subtags.iter().all(|subtag| greater_tag_all_subtags.contains(subtag))
        && lesser_tag_all_supertags.iter().all(|supertag| greater_tag_all_supertags.contains(supertag))
}

#[cfg(test)]
mod tests {
    use crate::{common::consensus::{proposal::ItemType, stability::Stability}, endpoints::tag::proposal::propose::interpreter::validate_topology::can_relate_by_inclusion};

    #[test]
    fn test_can_relate_by_inclusion() {
        assert!(can_relate_by_inclusion(Some((ItemType::Proposal, Stability::Stable))));
        assert!(!can_relate_by_inclusion(Some((ItemType::Proposal, Stability::Unstable))));
        
        // 提案
        assert!(!can_relate_by_inclusion(Some((ItemType::Reachable, Stability::Stable))));
        assert!(!can_relate_by_inclusion(Some((ItemType::Reachable, Stability::Unstable))));
        assert!(can_relate_by_inclusion(None));
    }
}