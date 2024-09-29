use std::collections::HashSet;

use bb8_redis::{bb8::PooledConnection, RedisConnectionManager};
use redis::cmd;

use crate::{common::{fallible::Fallible, tag::{non_top_tag_id::NonTopTagId, tag_id::TagId}}, endpoints::tag::proposal::propose::dsl::validate_topology::{ValidateTopology, ValidateTopologyError}, helper::redis::{conn, namespace::Namespace, namespace::NAMESPACE_SEPARATOR, SUBTAGS_NAMESPACE, SUPERTAGS_NAMESPACE}};

use super::ProposeTagRelationImpl;

impl ValidateTopology for ProposeTagRelationImpl {
    async fn is_acyclic(&self, subtag_id: NonTopTagId, supertag_id: NonTopTagId) -> Fallible<bool, ValidateTopologyError> {
        let mut conn = conn(&self.cache, |e| ValidateTopologyError::IsAcyclicFailed(e.into())).await?;
        
        cmd("ZSCORE")
            .arg(format!("{}{}{}", SUBTAGS_NAMESPACE, NAMESPACE_SEPARATOR, subtag_id))
            .arg(supertag_id)
            .query_async::<Option<()>>(&mut *conn)
            .await
            .map(|v| v.is_none())
            .map_err(|e| ValidateTopologyError::IsAcyclicFailed(e.into()))
    }

    async fn is_equivalent(&self, lesser_tag_id: NonTopTagId, greater_tag_id: NonTopTagId) -> Fallible<bool, ValidateTopologyError> {
        let mut conn = conn(&self.cache, |e| ValidateTopologyError::IsEquivalentFailed(e.into())).await?;
        
        async fn fetch_related_tags(conn: &mut PooledConnection<'_, RedisConnectionManager>, namespace: Namespace, tag_id: NonTopTagId) -> Fallible<HashSet<TagId>, ValidateTopologyError> {
            cmd("ZRANGE")
            .arg(format!("{}{}{}", namespace, NAMESPACE_SEPARATOR, tag_id))
            .arg(0)
            .arg(-1)
            .query_async::<HashSet<TagId>>(&mut **conn)
            .await
            .map_err(|e| ValidateTopologyError::IsEquivalentFailed(e.into()))
        }

        let mut lesser_tag_subtags = fetch_related_tags(&mut conn, SUBTAGS_NAMESPACE, lesser_tag_id).await?;
        let mut lesser_tag_supertags = fetch_related_tags(&mut conn, SUPERTAGS_NAMESPACE, lesser_tag_id).await?;
        let mut greater_tag_subtags = fetch_related_tags(&mut conn, SUBTAGS_NAMESPACE, greater_tag_id).await?;
        let mut greater_tag_supertags = fetch_related_tags(&mut conn, SUPERTAGS_NAMESPACE, greater_tag_id).await?;

        if lesser_tag_subtags.len() >= greater_tag_subtags.len() {
            std::mem::swap(&mut lesser_tag_subtags, &mut greater_tag_subtags);
            std::mem::swap(&mut lesser_tag_supertags, &mut greater_tag_supertags);
        }

        let is_included = lesser_tag_subtags.iter().all(|subtag| greater_tag_subtags.contains(subtag))
            && lesser_tag_supertags.iter().all(|supertag| greater_tag_supertags.contains(supertag));

        Ok(is_included)
    }
}