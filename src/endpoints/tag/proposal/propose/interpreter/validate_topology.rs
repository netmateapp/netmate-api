use std::{collections::HashSet, sync::Arc};

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{consensus::{is_unstable_proposal, proposal::IsProposal, stability::Stability}, fallible::Fallible, tag::{non_top_tag::NonTopTagId, tag_id::TagId}}, endpoints::tag::proposal::propose::dsl::validate_topology::{ValidateTopology, ValidateTopologyError}};

use super::ProposeTagRelationImpl;

impl ValidateTopology for ProposeTagRelationImpl {
    async fn is_acyclic(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId) -> Fallible<bool, ValidateTopologyError> {
        let related_tag_status = self.db
            .execute_unpaged(&self.select_subtag, (subtag_id, supertag_id))
            .await
            .map_err(|e| ValidateTopologyError::IsAcyclicFailed(e.into()))?
            .maybe_first_row_typed::<(IsProposal, Stability)>()
            .map_err(|e| ValidateTopologyError::IsAcyclicFailed(e.into()))?;

        Ok(can_relate_by_inclusion(related_tag_status))
    }

    async fn is_equivalent(&self, lesser_tag_id: NonTopTagId, greater_tag_id: NonTopTagId) -> Fallible<bool, ValidateTopologyError> {
        async fn fetch_all_related_tags(db: &Arc<Session>, selector: &Arc<PreparedStatement>, tag_id: NonTopTagId) -> Fallible<HashSet<TagId>, ValidateTopologyError> {
            db.execute_unpaged(selector, (tag_id, ))
            .await
            .map_err(|e| ValidateTopologyError::IsEquivalentFailed(e.into()))?
            .rows_typed()
            .map(|rows| {
                rows.flatten()
                    .filter(|(_, is_proposal, is_stable): &(TagId, IsProposal, Stability)| !is_unstable_proposal(*is_proposal, *is_stable))
                    .map(|(tag_id, _, _)| tag_id)
                    .collect::<HashSet<TagId>>()
            })
            .map_err(|e| ValidateTopologyError::IsEquivalentFailed(e.into()))
        }

        let lesser_tag_all_subtags = fetch_all_related_tags(&self.db, &self.select_all_subtag, lesser_tag_id).await?;
        let lesser_tag_all_supertags = fetch_all_related_tags(&self.db, &self.select_all_supertag, lesser_tag_id).await?;
        let greater_tag_all_subtags = fetch_all_related_tags(&self.db, &self.select_all_subtag, greater_tag_id).await?;
        let greater_tag_all_supertags = fetch_all_related_tags(&self.db, &self.select_all_supertag, greater_tag_id).await?;

        Ok(can_relate_by_equivalence(&lesser_tag_all_subtags, &lesser_tag_all_supertags, &greater_tag_all_subtags, &greater_tag_all_supertags))
    }
}

fn can_relate_by_inclusion(related_tag_status: Option<(IsProposal, Stability)>) -> bool {
    match related_tag_status {
        Some((is_proposal, is_stable)) => is_unstable_proposal(is_proposal, is_stable),
        None => true
    }
}

fn can_relate_by_equivalence<'a>(mut lesser_tag_all_subtags: &'a HashSet<TagId>, mut lesser_tag_all_supertags: &'a HashSet<TagId>, mut greater_tag_all_subtags: &'a HashSet<TagId>, mut greater_tag_all_supertags: &'a HashSet<TagId>) -> bool {
    // 比較を容易にするために、サイズの小さい方をlesser_tag_xxxに入れる
    if lesser_tag_all_subtags.len() >= greater_tag_all_subtags.len() {
        std::mem::swap(&mut lesser_tag_all_subtags, &mut greater_tag_all_subtags);
        std::mem::swap(&mut lesser_tag_all_supertags, &mut greater_tag_all_supertags);
    }
    
    // 完全に包含されている場合は同値関係を形成できる
    is_subset(lesser_tag_all_subtags, greater_tag_all_subtags) && is_subset(lesser_tag_all_supertags, greater_tag_all_supertags)
}

fn is_subset(lesser: &HashSet<TagId>, greater: &HashSet<TagId>) -> bool {
    lesser.iter().all(|tag| greater.contains(tag))
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use crate::{common::{consensus::{proposal::IsProposal, stability::Stability}, tag::tag_id::TagId}, endpoints::tag::proposal::propose::interpreter::validate_topology::{can_relate_by_equivalence, can_relate_by_inclusion}, helper::test::mock_tag_id};

    #[test]
    fn test_can_relate_by_inclusion() {
        assert!(can_relate_by_inclusion(Some((IsProposal::Proposal, Stability::Unstable))));
        assert!(can_relate_by_inclusion(None));

        // 安定的な提案である場合は、包含関係を定義できない
        assert!(!can_relate_by_inclusion(Some((IsProposal::Proposal, Stability::Stable))));
        
        // 安定性は提案固有のパラメータであるため、提案ではないものには関係ない
        // 以下の2つの条件は実際には存在しない
        assert!(!can_relate_by_inclusion(Some((IsProposal::NotProposal, Stability::Stable))));
        assert!(!can_relate_by_inclusion(Some((IsProposal::NotProposal, Stability::Unstable))));
    }

    static TAG1: LazyLock<TagId> = LazyLock::new(|| mock_tag_id(0));
    static TAG2: LazyLock<TagId> = LazyLock::new(|| mock_tag_id(1));
    static TAG3: LazyLock<TagId> = LazyLock::new(|| mock_tag_id(2));
    static TAG4: LazyLock<TagId> = LazyLock::new(|| mock_tag_id(3));

    #[test]
    fn test_can_relate_by_equivalence() {
        use std::collections::HashSet;

        let mut lesser_tag_all_subtags = HashSet::new();
        lesser_tag_all_subtags.insert(*TAG1);
        
        let mut lesser_tag_all_supertags = HashSet::new();
        lesser_tag_all_supertags.insert(*TAG3);

        let mut greater_tag_all_subtags = HashSet::new();
        greater_tag_all_subtags.insert(*TAG1);
        greater_tag_all_subtags.insert(*TAG2);

        let mut greater_tag_all_supertags = HashSet::new();
        greater_tag_all_supertags.insert(*TAG3);
        greater_tag_all_supertags.insert(*TAG4);
        assert!(can_relate_by_equivalence(&lesser_tag_all_subtags, &lesser_tag_all_supertags, &greater_tag_all_subtags, &greater_tag_all_supertags));

        greater_tag_all_subtags.remove(&TAG1);
        assert!(!can_relate_by_equivalence(&lesser_tag_all_subtags, &lesser_tag_all_supertags, &greater_tag_all_subtags, &greater_tag_all_supertags));

        // lesser_tag_all_supertagsがgreater_tag_all_supertagsに含まれていない場合は同値関係を形成できない
        greater_tag_all_subtags.insert(*TAG1);
        greater_tag_all_supertags.remove(&TAG3);
        assert!(!can_relate_by_equivalence(&lesser_tag_all_subtags, &lesser_tag_all_supertags, &greater_tag_all_subtags, &greater_tag_all_supertags));
    }

}