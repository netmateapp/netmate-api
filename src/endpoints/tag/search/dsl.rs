use std::collections::HashMap;

use thiserror::Error;

use crate::common::{fallible::Fallible, tag::{hierarchy::TagHierarchy, language_group::LanguageGroup, tag_id::TagId, tag_info::TagInfo, tag_name::TagName}};

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
        
        // タグ情報をフェッチ
        let tag_info_map = self
            .fetch_tag_info(tag_id, related_tag_id, hierarchy, matched_tag_ids)
            .await?;
        
        // TagInfoのリストを構築
        let tag_infos = matched_tags
            .into_iter()
            .map(|(tag_id, tag_name)| {
                // タグ情報が存在しない場合は (false, false) を使用
                let (is_proposal, is_stable) = tag_info_map.get(&tag_id)
                    .copied()
                    .unwrap_or((false, false));
                
                TagInfo::new(tag_id, tag_name, is_proposal, is_stable)
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

    // (bool, bool) は is_proposal, is_stable
    async fn fetch_tag_info(
        &self,
        tag_id: TagId,
        related_tag_id: TagId,
        hierarchy: TagHierarchy,
        tags: Vec<TagId>
    ) -> Fallible<HashMap<TagId, (bool, bool)>, SearchWithinHierarchicalTagListError>;
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