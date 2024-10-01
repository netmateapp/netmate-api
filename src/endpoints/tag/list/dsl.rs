use serde::{ser::SerializeStruct, Serialize};
use thiserror::Error;

use crate::common::{fallible::Fallible, page::ZeroBasedPage, tag::{hierarchy::TagHierarchy, tag_id::TagId, tag_name::TagName}};

pub(crate) trait ListRelatedTags {
    async fn list_related_tags(&self, tag_id: TagId, relationship: TagHierarchy, page: ZeroBasedPage) -> Fallible<Vec<TagInfo>, ListRelatedTagsError>;
}

#[derive(Debug, Error)]
pub enum ListRelatedTagsError {
    #[error("タグリストの取得に失敗しました")]
    ListRelatedTagsFailed(#[source] anyhow::Error),
}

pub struct TagInfo {
    id: TagId,
    name: TagName,
    is_proposal: bool,
    is_stable: bool,
}

impl TagInfo {
    pub fn new(id: TagId, name: TagName, is_proposal: bool, is_stable: bool,) -> Self {
        TagInfo { id, name, is_proposal, is_stable }
    }

    pub fn id(&self) -> &TagId {
        &self.id
    }

    pub fn name(&self) -> &TagName {
        &self.name
    }

    pub fn is_proposal(&self) -> bool {
        self.is_proposal
    }

    pub fn is_stable(&self) -> bool {
        self.is_stable
    }
}

impl Serialize for TagInfo {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("TagInfo", 4)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("is_proposal", &self.is_proposal)?;
        state.serialize_field("is_stable", &self.is_stable)?;
        state.end()
    }
}
