use serde::{de, Deserialize, Deserializer};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TagHierarchy {
    Super = 0,
    Equivalent = 1,
    Sub = 2,
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

impl<'de> Deserialize<'de> for TagHierarchy {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u8::deserialize(deserializer)
            .and_then(|value| TagHierarchy::try_from(value).map_err(de::Error::custom))
    }
}