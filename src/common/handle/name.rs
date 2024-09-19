use std::str::FromStr;

use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::common::character_count::calculate_character_cost;

const HANDLE_NAME_MAX_CHARACTER_COST: usize = 100;

#[derive(Debug)]
pub struct HandleName(String);

impl HandleName {
    pub fn value(&self) -> &String {
        &self.0
    }
}

impl FromStr for HandleName {
    type Err = ParseHandleNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseHandleNameError::Empty);
        }

        // 文字コスト計算はO(n)であるため、非常に長い文字列は事前に除外する
        // CJK範囲の文字のみで構成された名義の最大長は、`HANDLE_NAME_MAX_CHARACTER_COST / 2` であり、
        // 有効な名義のうち最もbyte数の多いものは、CJK統合拡張漢字B～F範囲の文字(4byte)のみで構成される
        // よって、`HANDLE_NAME_MAX_CHARACTER_COST / 2 cost * 4 bytes`
        //  = `HANDLE_NAME_MAX_CHARACTER_COST * 2` よりbyte数の多い文字列は名義になり得ないことが保証される
        if s.len() > HANDLE_NAME_MAX_CHARACTER_COST * 2 {
            return Err(ParseHandleNameError::CharacterCostOverflow);
        }

        if calculate_character_cost(s) > HANDLE_NAME_MAX_CHARACTER_COST {
            return Err(ParseHandleNameError::CharacterCostOverflow);
        }

        Ok(HandleName(String::from(s)))
    }
}

#[derive(Debug, Error)]
pub enum ParseHandleNameError {
    #[error("空文字は許可されていません")]
    Empty,
    #[error("文字数が多すぎます")]
    CharacterCostOverflow,
}

impl Serialize for HandleName {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(&self.value(), serializer)
    }
}

impl<'de> Deserialize<'de> for HandleName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)
            .and_then(|v| HandleName::from_str(v.as_str()).map_err(de::Error::custom))
    }
}

impl SerializeValue for HandleName {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&self.0, typ, writer)
    }
}