use std::collections::HashMap;

use serde::{ser::SerializeStruct, Serialize};
use thiserror::Error;

use crate::common::{consensus::{proposal::IsProposal, stability::Stability}, fallible::Fallible, tag::{hierarchy::TagHierarchy, language_group::LanguageGroup, tag_id::TagId, tag_name::TagName}};

// search_afterを使う
pub(crate) trait SearchWithinHierarchicalTagList {
    async fn search_within_hierarchical_tag_list(
        &self,
        query: TagName,
        language_group: LanguageGroup,
        search_after: Option<TagId>,
        tag_id: TagId,
        related_tag_id: TagId,
        hierarchy: TagHierarchy
    ) -> Fallible<Vec<TagInfo>, SearchWithinHierarchicalTagListError> {
        // マッチするタグを検索
        let matched_tags = self
            .search_matched_tags(query, language_group, search_after)
            .await?;
        
        // マッチしたタグのIDを抽出
        let matched_tag_ids: Vec<TagId> = matched_tags.iter().map(|(tag_id, _)| *tag_id).collect();

        if matched_tag_ids.is_empty() {
            return Ok(Vec::new());
        }
        
        // タグ情報をフェッチ
        let tag_info_map = self
            .fetch_tag_info(tag_id, related_tag_id, hierarchy, matched_tag_ids)
            .await?;
        
        // TagInfoのリストを構築
        let tag_infos = matched_tags
            .into_iter()
            .map(|(tag_id, tag_name)| {
                match tag_info_map.get(&tag_id) {
                    Some((is_proposal, stability)) => TagInfo::new(tag_id, tag_name, true, *is_proposal, *stability),
                    None => TagInfo::new(tag_id, tag_name, false, IsProposal::NotProposal, Stability::Unstable)
                }
            })
            .collect::<Vec<TagInfo>>();
        
        Ok(tag_infos)
    }

    async fn search_matched_tags(
        &self,
        query: TagName,
        language_group: LanguageGroup,
        search_after: Option<TagId>
    ) -> Fallible<Vec<(TagId, TagName)>, SearchWithinHierarchicalTagListError>;

    // tagsは空ではないことが保証されている
    async fn fetch_tag_info(
        &self,
        tag_id: TagId,
        related_tag_id: TagId,
        hierarchy: TagHierarchy,
        tags: Vec<TagId>
    ) -> Fallible<HashMap<TagId, (IsProposal, Stability)>, SearchWithinHierarchicalTagListError>;
}

#[derive(Debug, Error)]
pub enum SearchWithinHierarchicalTagListError {
    #[error("タグの検索に失敗しました")]
    SearchMatchedTags(#[source] anyhow::Error),
    #[error("階層におけるタグ情報の取得に失敗しました")]
    FetchTagInfoFailed(#[source] anyhow::Error),
    #[error("階層別タグ一覧内の検索に失敗しました")]
    SearchWithinHierarchicalTagListFailed(#[source] anyhow::Error),
}

pub struct TagInfo {
    id: TagId,
    name: TagName,
    is_reachable: bool,
    is_proposal: IsProposal,
    is_stable: Stability,
}

impl TagInfo {
    pub fn new(id: TagId, name: TagName, is_reachable: bool, is_proposal: IsProposal, is_stable: Stability) -> Self {
        TagInfo { id, name, is_reachable, is_proposal, is_stable }
    }

    pub fn id(&self) -> &TagId {
        &self.id
    }

    pub fn name(&self) -> &TagName {
        &self.name
    }

    pub fn is_reachable(&self) -> bool {
        self.is_reachable
    }

    pub fn is_proposal(&self) -> IsProposal {
        self.is_proposal
    }

    pub fn is_stable(&self) -> Stability {
        self.is_stable
    }
}

impl Serialize for TagInfo {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("TagInfo", 5)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("is_reachable", &self.is_reachable)?;
        state.serialize_field("is_proposal", &bool::from(self.is_proposal()))?;
        state.serialize_field("is_stable", &bool::from(self.is_stable()))?;
        state.end()
    }
}