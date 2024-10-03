use std::{fmt::{self, Display}, str::FromStr};

use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::common::character_count::calculate_character_cost;

const MAX_TAG_NAME_CHARACTER_COST: usize = 100;

pub struct TagName(String);

impl TagName {
    pub fn value(&self) -> &String {
        &self.0
    }
}

impl FromStr for TagName {
    type Err = ParseTagNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if calculate_character_cost(s) <= MAX_TAG_NAME_CHARACTER_COST {
            Ok(TagName(s.to_string()))
        } else {
            Err(ParseTagNameError)
        }
    }
}

impl Display for TagName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[derive(Debug, Error)]
#[error("タグ名の解析に失敗しました")]
pub struct ParseTagNameError;

impl<'de> Deserialize<'de> for TagName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)
            .and_then(|value| TagName::from_str(value.as_str()).map_err(de::Error::custom))
    }
}

impl Serialize for TagName {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(&self.value(), serializer)
    }
}

impl SerializeValue for TagName {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(self.value(), typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for TagName {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        String::from_cql(cql_val)
            .and_then(|v| TagName::from_str(v.as_str()).map_err(|_| FromCqlValError::BadVal))
    }
}