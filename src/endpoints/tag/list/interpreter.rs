use std::{str::FromStr, sync::Arc};

use redis::cmd;
use uuid::Uuid;

use crate::{common::{fallible::Fallible, page::ZeroBasedPage, tag::{redis_tag_info::RedisTagInfo, relationship::TagRelationType, tag_id::TagId, tag_name::TagName}, uuid::uuid4::Uuid4}, endpoints::tag::list::dsl::TagInfo, helper::{error::InitError, redis::{connection::{conn, Pool}, namespace::NAMESPACE_SEPARATOR, namespaces::{EQUIVALENT, SUB, SUPER, TAG_LIST}}}};

use super::dsl::{ListRelatedTags, ListRelatedTagsError};

pub struct ListRelatedTagsImpl {
    cache: Arc<Pool>,
}

impl ListRelatedTagsImpl {
    pub async fn try_new(cache: Arc<Pool>) -> Result<Self, InitError<Self>> {
        Ok(Self { cache })
    }
}

impl ListRelatedTags for ListRelatedTagsImpl {
    async fn list_related_tags(&self, tag_id: TagId, relationship: TagRelationType, page: ZeroBasedPage) -> Fallible<Vec<TagInfo>, ListRelatedTagsError> {
        let namespace = match relationship {
            TagRelationType::Super => SUPER,
            TagRelationType::Equivalent => EQUIVALENT,
            TagRelationType::Sub => SUB,
        };

        const PAGE_SIZE: u32 = 10;

        let mut conn = conn(&self.cache, |e| ListRelatedTagsError::ListRelatedTagsFailed(e.into())).await?;

        let transitive_closure_and_unstable_proposals = cmd("ZRANGE")
            .arg(format!("{}{}{}{}{}", TAG_LIST, NAMESPACE_SEPARATOR, tag_id, NAMESPACE_SEPARATOR, namespace))
            .arg(page.first_index(PAGE_SIZE))
            .arg(page.last_index(PAGE_SIZE))
            .arg("REV")
            .arg("WITHSCORES")
            .query_async::<Vec<(String, RedisTagInfo)>>(&mut *conn) // メンバー, スコア の順で返される
            .await
            .map_err(|e| ListRelatedTagsError::ListRelatedTagsFailed(e.into()))?
            .iter()
            .map(|(id_and_name, info)| {
                let mut split = id_and_name.splitn(2, '$');
                let id = split.next()
                    .map(|s| Uuid::from_str(s).unwrap())
                    .map(|uuid | TagId::of(Uuid4::try_from(uuid).unwrap()))
                    .unwrap();

                let name = split.next()
                    .map(|s| TagName::from_str(s).unwrap())
                    .unwrap();

                TagInfo::new(id, name, info.is_proposal())
            }).collect::<Vec<TagInfo>>();

            Ok(transitive_closure_and_unstable_proposals)
    }
}