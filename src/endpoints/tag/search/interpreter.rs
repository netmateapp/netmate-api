use std::{collections::HashMap, str::FromStr, sync::Arc};

use elasticsearch::{Elasticsearch, SearchParts};
use scylla::{prepared_statement::PreparedStatement, Session};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{common::{consensus::{proposal::IsProposal, stability::Stability}, fallible::Fallible, tag::{hierarchy::TagHierarchy, language_group::LanguageGroup, tag_id::TagId, tag_name::TagName}, uuid::uuid4::Uuid4}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{SearchWithinHierarchicalTagList, SearchWithinHierarchicalTagListError};

pub struct SearchWithinHierarchicalTagListImpl {
    db: Arc<Session>,
    client: Arc<Elasticsearch>,
    select_tags_info: Arc<PreparedStatement>,
}

impl SearchWithinHierarchicalTagListImpl {
    pub async fn try_new(db: Arc<Session>, client: Arc<Elasticsearch>) -> Result<Self, InitError<Self>> {
        let select_tags_info = prepare(&db, "SELECT related_tag_id, is_proposal, is_stable FROM hierarchical_tag_lists WHERE tag_id = ? AND hierarchy = ? AND related_tag_id IN (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)").await?;

        Ok(Self { db, client, select_tags_info })
    }
}

const PAGE_SIZE: usize = 10;

impl SearchWithinHierarchicalTagList for SearchWithinHierarchicalTagListImpl {
    async fn search_matched_tags(&self, query: &TagName, language_group: LanguageGroup, search_after: &Option<TagId>) -> Fallible<Vec<(TagId, TagName)>, SearchWithinHierarchicalTagListError> {
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
            "size": PAGE_SIZE
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

    async fn fetch_tag_info(&self, tag_id: TagId, hierarchy: TagHierarchy, mut tags: Vec<TagId>) -> Fallible<HashMap<TagId, (IsProposal, Stability)>, SearchWithinHierarchicalTagListError> {
        extend_to_length(&mut tags);

        fn get(tags: &[TagId], i: usize) -> TagId {
          *tags.get(i).unwrap()
        }

        let tag_infos: HashMap<TagId, (IsProposal, Stability)> = self.db
            .execute_unpaged(&self.select_tags_info, (tag_id, hierarchy, get(&tags, 0), get(&tags, 1), get(&tags, 2), get(&tags, 3), get(&tags, 4), get(&tags, 5), get(&tags, 6), get(&tags, 7), get(&tags, 8), get(&tags, 9)))
            .await
            .map_err(|e| SearchWithinHierarchicalTagListError::FetchTagInfoFailed(e.into()))?
            .rows_typed()
            .map_err(|e| SearchWithinHierarchicalTagListError::FetchTagInfoFailed(e.into()))?
            .flatten()
            .map(|(tag_id, is_proposal, stability)| (tag_id, (is_proposal, stability)))
            .collect();

        Ok(tag_infos)
    }
}

// tagsが空ではない前提
fn extend_to_length(tags: &mut Vec<TagId>) {
  if tags.len() < PAGE_SIZE {
    // tagsの要素が10個未満の場合は、最後の要素で埋める
    let last_tag_id = *tags.last().unwrap();
    let diff = PAGE_SIZE - tags.len();
    for _ in 0..diff {
        tags.push(last_tag_id);
    }
  }
}

#[cfg(test)]
mod tests {
    use crate::{common::tag::tag_id::TagId, endpoints::tag::search::interpreter::extend_to_length};

    use super::PAGE_SIZE;

    #[test]
    fn test_extend_to_length() {
        for i in 0..PAGE_SIZE {
            let mut vec = vec![];
            for _ in 0..=i {
                vec.push(TagId::gen());
            }
            extend_to_length(&mut vec);
            assert_eq!(vec.len(), 10);
        }
    }
}