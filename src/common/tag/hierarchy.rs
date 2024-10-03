use std::fmt::{self, Display};

use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{de, Deserialize, Deserializer};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TagHierarchy {
    Super = 0,
    Equivalent = 1,
    Sub = 2,
}

impl From<TagHierarchy> for u8 {
    fn from(value: TagHierarchy) -> Self {
        value as u8
    }
}

impl From<TagHierarchy> for i8 {
    fn from(value: TagHierarchy) -> Self {
        u8::from(value) as i8
    }
}

impl TryFrom<u8> for TagHierarchy {
    type Error = ParseTagRelationTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TagHierarchy::Super),
            1 => Ok(TagHierarchy::Equivalent),
            2 => Ok(TagHierarchy::Sub),
            _ => Err(ParseTagRelationTypeError),
        }
    }
}

#[derive(Debug, Error)]
#[error("タグ階層の解析に失敗しました")]
pub struct ParseTagRelationTypeError;

impl Display for TagHierarchy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TagHierarchy::Super => "上位",
            TagHierarchy::Equivalent => "同値",
            TagHierarchy::Sub => "下位"
        };
        write!(f, "{}", s)
    }
}

impl<'de> Deserialize<'de> for TagHierarchy {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u8::deserialize(deserializer)
            .and_then(|value| TagHierarchy::try_from(value).map_err(de::Error::custom))
    }
}

impl SerializeValue for TagHierarchy {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&i8::from(*self), typ, writer)
    }
}