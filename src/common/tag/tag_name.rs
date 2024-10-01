use std::{fmt::{self, Display}, str::FromStr};

use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
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