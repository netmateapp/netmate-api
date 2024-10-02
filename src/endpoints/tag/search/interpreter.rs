use std::{collections::HashMap, str::FromStr, sync::Arc};

use elasticsearch::{Elasticsearch, SearchParts};
use scylla::{prepared_statement::PreparedStatement, Session};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{common::{fallible::Fallible, tag::{hierarchy::TagHierarchy, language_group::LanguageGroup, tag_id::TagId, tag_info::TagInfo, tag_name::TagName}, uuid::uuid4::Uuid4}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{SearchWithinHierarchicalTagList, SearchWithinHierarchicalTagListError};

pub struct SearchWithinHierarchicalTagListImpl {
    db: Arc<Session>,
    client: Arc<Elasticsearch>,
    select_tags_info: Arc<PreparedStatement>,
}

impl SearchWithinHierarchicalTagListImpl {
    pub async fn try_new(&self, db: Arc<Session>, client: Arc<Elasticsearch>) -> Result<Self, InitError<Self>> {
        let select_tags_info = prepare(&db, "SELECT related_tag_id, is_unstable_proposal, is_status_calculated FROM hierarchical_tag_lists WHERE tag_id = ? AND relation = ? AND related_tag_id IN (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)").await?;

        Ok(Self { db, client, select_tags_info })
    }
}

impl SearchWithinHierarchicalTagList for SearchWithinHierarchicalTagListImpl {
    async fn search_matched_tags(&self, query: TagName, language_group: LanguageGroup, search_after: Option<TagId>) -> Fallible<Vec<(TagId, TagName)>, SearchWithinHierarchicalTagListError> {
        let mut search_body = json!({
            "query": {
              "bool": {
                "must": [
                  {
                    "match": {
                      "name": query
                    }
                  }
                ],
                "filter": [
                  {
                    "term": {
                      "language": u8::from(language_group)
                    }
                  }
                ]
              }
            },
            "sort": [
              { "tag_id": "asc" }
            ],
            "_source": ["id", "name"],
            "size": 10
        });

        // search_afterがある場合は、それをクエリに追加
        if let Some(search_after) = search_after {
            let query_map = search_body.as_object_mut().unwrap();
            query_map.insert("search_after".to_string(), Value::Array(vec![Value::String(search_after.to_string())]));
        }

        let response = self.client
            .search(SearchParts::Index(&["tags"]))
            .body(search_body)
            .send()
            .await
            .map_err(|e| SearchWithinHierarchicalTagListError::SearchMatchedTags(e.into()))?;

        let response_body: Value = response.json()
            .await
            .map_err(|e| SearchWithinHierarchicalTagListError::SearchMatchedTags(e.into()))?;
        
        let mut matched_tags: Vec<(TagId, TagName)> = Vec::new();

        if let Some(hits) = response_body["hits"]["hits"].as_array() {
            for hit in hits {
                if let (Some(tag_id), Some(tag_name)) = (
                    hit["_source"]["id"].as_str(),
                    hit["_source"]["name"].as_str()
                ) {
                    let tag_id = TagId::of(Uuid4::try_from(Uuid::parse_str(tag_id).unwrap()).unwrap());
                    let tag_name = TagName::from_str(tag_name).unwrap();
                    matched_tags.push((tag_id, tag_name));
                }
            }
        }

        Ok(matched_tags)
    }

    async fn fetch_tag_info(&self, tag_id: TagId, related_tag_id: TagId, hierarchy: TagHierarchy, tags: Vec<TagId>) -> Fallible<HashMap<TagId, (bool, bool)>, SearchWithinHierarchicalTagListError> {
        if tags.len() < 10 {
            // tagsの要素が10個未満の場合は、最後の要素で埋める
            let last_tag_id = tags.last().unwrap();
            let diff = 10 - tags.len();
            let mut tags = tags.clone();
            for _ in 0..diff {
                tags.push(*last_tag_id);
            }
        }

        Ok(HashMap::new())
    }
}