use std::fmt::{self, Display};

use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{de, Deserialize, Deserializer};
use thiserror::Error;

use super::{tag_id::TagId, top_tag_id::is_top_tag_id};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NonTopTagId(TagId);

impl NonTopTagId {
    pub fn gen() -> NonTopTagId {
        // 奇跡が起きない限りO(1)で終了する
        loop {
            match NonTopTagId::try_from(TagId::gen()) {
                Ok(non_top_tag_id) => return non_top_tag_id,
                _ => (),
            }
        }
    }

    pub fn value(&self) -> TagId {
        self.0
    }
}

#[derive(Debug, Error, PartialEq)]
#[error("非トップタグIDの解析に失敗しました")]
pub struct ParseNonTopTagIdError;

impl TryFrom<TagId> for NonTopTagId {
    type Error = ParseNonTopTagIdError;

    fn try_from(value: TagId) -> Result<Self, Self::Error> {
        if is_top_tag_id(value) {
            Err(ParseNonTopTagIdError)
        } else {
            Ok(Self(value))
        }
    }
}

impl Display for NonTopTagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl<'de> Deserialize<'de> for NonTopTagId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        TagId::deserialize(deserializer)
            .and_then(|v| NonTopTagId::try_from(v).map_err(de::Error::custom))
    }
}

impl SerializeValue for NonTopTagId {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&self.value(), typ, writer)
    }
}