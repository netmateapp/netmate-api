use std::str::FromStr;

use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::common::character_count::calculate_character_cost;

const HANDLE_NAME_MAX_CHARACTER_COST: usize = 100;

#[derive(Debug, Serialize, Deserialize)]
pub struct HandleName(String);

impl HandleName {
    pub fn value(&self) -> &String {
        &self.0
    }
}

impl FromStr for HandleName {
    type Err = ParseHandleNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 長い文字列は文字コストを計算する前に弾く
        // CJK範囲の文字のみで構成された名義の最大長は、`HANDLE_NAME_MAX_CHARACTER_COST / 2` となる
        // 有効な名義のうち最もbyte数の多いものは、CJK統合拡張漢字B～F範囲の文字(4byte)のみで構成される
        // `HANDLE_NAME_MAX_CHARACTER_COST / 2 cost * 4 bytes` = `HANDLE_NAME_MAX_CHARACTER_COST * 2`
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
    #[error("文字数が多すぎます")]
    CharacterCostOverflow,
}

impl SerializeValue for HandleName {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&self.0, typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for HandleName {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        String::from_cql(cql_val).map(HandleName)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonAnonymousHandleName(String);

impl TryFrom<HandleName> for NonAnonymousHandleName {
    type Error = ParseNonAnonymousHandleNameError;

    fn try_from(value: HandleName) -> Result<Self, Self::Error> {
        if value.value().is_empty() {
            return Err(ParseNonAnonymousHandleNameError::EmptyHandleName);
        }

        Ok(NonAnonymousHandleName(value.0))
    }
}

#[derive(Debug, Error)]
pub enum ParseNonAnonymousHandleNameError {
    #[error("空文字は匿名名義のみに許可されています")]
    EmptyHandleName,
}

impl SerializeValue for NonAnonymousHandleName {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&self.0, typ, writer)
    }
}