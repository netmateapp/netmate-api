use serde::{de, Deserialize, Deserializer};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TagRelationType {
    Super = 0,
    Equivalent = 1,
    Sub = 2,
}

impl TryFrom<u8> for TagRelationType {
    type Error = ParseTagRelationTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TagRelationType::Super),
            1 => Ok(TagRelationType::Equivalent),
            2 => Ok(TagRelationType::Sub),
            _ => Err(ParseTagRelationTypeError),
        }
    }
}

#[derive(Debug, Error)]
#[error("タグ関係の種類の解析に失敗しました")]
pub struct ParseTagRelationTypeError;

impl<'de> Deserialize<'de> for TagRelationType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u8::deserialize(deserializer)
            .and_then(|value| TagRelationType::try_from(value).map_err(de::Error::custom))
    }
}