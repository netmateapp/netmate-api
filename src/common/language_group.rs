use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use thiserror::Error;

use super::language::Language;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum LanguageGroup {
    Japanese = 0,
    Korean = 1,
    TaiwaneseMandarin = 2,
    English = 3,
}

impl LanguageGroup {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

impl From<Language> for LanguageGroup {
    fn from(value: Language) -> Self {
        match value {
            Language::Japanese => LanguageGroup::Japanese,
            Language::Korean => LanguageGroup::Korean,
            Language::TaiwaneseMandarin => LanguageGroup::TaiwaneseMandarin,
            Language::AmericanEnglish => LanguageGroup::English,
        }
    }
}

impl From<LanguageGroup> for u8 {
    fn from(value: LanguageGroup) -> Self {
        value.as_u8()
    }
}

impl From<LanguageGroup> for i8 {
    fn from(value: LanguageGroup) -> Self {
        u8::from(value) as i8
    }
}

#[derive(Debug, Error)]
#[error("言語グループの解析に失敗しました")]
pub struct ParseLanguageGroupError;

impl TryFrom<u8> for LanguageGroup {
    type Error = ParseLanguageGroupError;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let language_group = match value {
            0 => LanguageGroup::Japanese,
            1 => LanguageGroup::Korean,
            2 => LanguageGroup::TaiwaneseMandarin,
            3 => LanguageGroup::English,
            _ => return Err(ParseLanguageGroupError)
        };
        Ok(language_group)
    }
}

impl TryFrom<i8> for LanguageGroup {
    type Error = ParseLanguageGroupError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        LanguageGroup::try_from(value as u8)
    }
}

impl SerializeValue for LanguageGroup {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        i8::from(*self).serialize(typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for LanguageGroup {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        i8::from_cql(cql_val)
            .and_then(|v| LanguageGroup::try_from(v).map_err(|_| FromCqlValError::BadVal))
    }
}
