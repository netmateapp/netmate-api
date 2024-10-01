use crate::{common::{fallible::Fallible, tag::{non_top_tag::NonTopTagId, redis_tag_info::{RedisTagInfo, TagListOrder}, tag_name::TagName}}, endpoints::tag::proposal::propose::dsl::update_tag_list::{UpdateTagRelationList, UpdateTagRelationListError}, helper::redis::{connection::conn, namespace::NAMESPACE_SEPARATOR, namespaces::{EQUIVALENT, SUB, SUPER, TAG_LIST}}};

use super::ProposeTagRelationImpl;

impl UpdateTagRelationList for ProposeTagRelationImpl {
    async fn update_inclusion_relation_list(&self, subtag_id: NonTopTagId, subtag_name: TagName, supertag_id: NonTopTagId, supertag_name: TagName) -> Fallible<(), UpdateTagRelationListError> {
        let mut conn = conn(&self.cache, |e| UpdateTagRelationListError::UpdateInclusionRelationListFailed(e.into())).await?;
        
        // タグリスト用のRedisデータに未安定の提案として追加
        self.insert_unstable_proposals_to_list
            .key(format!("{}{}{}{}{}", TAG_LIST, NAMESPACE_SEPARATOR, subtag_id, NAMESPACE_SEPARATOR, SUPER))
            .arg(RedisTagInfo::construct(TagListOrder::ReachableTagOrValidProposalOrUncalcProposal, 1000, true, false, false))
            .arg(format!("{}${}", supertag_id, supertag_name))
            .key(format!("{}{}{}{}{}", TAG_LIST, NAMESPACE_SEPARATOR, supertag_id, NAMESPACE_SEPARATOR, SUB))
            .arg(RedisTagInfo::construct(TagListOrder::ReachableTagOrValidProposalOrUncalcProposal, 1000, true, false, false))
            .arg(format!("{}${}", subtag_id, subtag_name))
            .invoke_async(&mut *conn)
            .await
            .map_err(|e| UpdateTagRelationListError::UpdateInclusionRelationListFailed(e.into()))?;

        self.db
            .execute_unpaged(&self.insert_inclusion_relation_proposal, (subtag_id, supertag_id, supertag_name, supertag_id, subtag_id, subtag_name))
            .await
            .map(|_| ())
            .map_err(|e| UpdateTagRelationListError::UpdateInclusionRelationListFailed(e.into()))
    }

    async fn update_equivalence_relation_list(&self, lesser_tag_id: NonTopTagId, lesser_tag_name: TagName, greater_tag_id: NonTopTagId, greater_tag_name: TagName) -> Fallible<(), UpdateTagRelationListError> {
        let mut conn = conn(&self.cache, |e| UpdateTagRelationListError::UpdateInclusionRelationListFailed(e.into())).await?;
        
        // タグリスト用のRedisデータに未安定の提案として追加
        self.insert_unstable_proposals_to_list
            .key(format!("{}{}{}{}{}", TAG_LIST, NAMESPACE_SEPARATOR, lesser_tag_id, NAMESPACE_SEPARATOR, EQUIVALENT))
            .arg(RedisTagInfo::construct(TagListOrder::ReachableTagOrValidProposalOrUncalcProposal, 1000, true, false, false))
            .arg(format!("{}${}", greater_tag_id, greater_tag_name))
            .key(format!("{}{}{}{}{}", TAG_LIST, NAMESPACE_SEPARATOR, greater_tag_id, NAMESPACE_SEPARATOR, EQUIVALENT))
            .arg(RedisTagInfo::construct(TagListOrder::ReachableTagOrValidProposalOrUncalcProposal, 1000, true, false, false))
            .arg(format!("{}${}", lesser_tag_id, lesser_tag_name))
            .invoke_async(&mut *conn)
            .await
            .map_err(|e| UpdateTagRelationListError::UpdateInclusionRelationListFailed(e.into()))?;

        self.db
            .execute_unpaged(&self.insert_equivalence_relation_proposal, (lesser_tag_id, greater_tag_id, greater_tag_name, greater_tag_id, lesser_tag_id, lesser_tag_name))
            .await
            .map(|_| ())
            .map_err(|e| UpdateTagRelationListError::UpdateInclusionRelationListFailed(e.into()))
    }
}