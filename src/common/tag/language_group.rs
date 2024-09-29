use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use thiserror::Error;

use crate::common::profile::language::Language;

use super::top_tag_id::{TopTagId, ENGLISH, JAPANESE, KOREAN, TAIWANESE_MANDARIN};

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

impl From<TopTagId> for LanguageGroup {
    fn from(value: TopTagId) -> Self {
        match value {
            JAPANESE => LanguageGroup::Japanese,
            KOREAN => LanguageGroup::Korean,
            TAIWANESE_MANDARIN => LanguageGroup::TaiwaneseMandarin,
            ENGLISH => LanguageGroup::English,
            _ => panic!("トップタグに対応する言語グループが指定されていません")
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

#[derive(Debug, Error, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::{LanguageGroup, ParseLanguageGroupError};

    #[test]
    fn try_from_valid_u8() {
        for i in 0u8..4 {
            let group = LanguageGroup::try_from(i);
            assert_eq!(group.map(u8::from), Ok(i))
        }
    }

    #[test]
    fn try_from_invalid_u8() {
        for i in 4u8..=u8::MAX {
            let group = LanguageGroup::try_from(i);
            assert_eq!(group.map(u8::from), Err(ParseLanguageGroupError))
        }
    }

    #[test]
    fn try_from_valid_i8() {
        for i in 0u8..4 {
            let group = LanguageGroup::try_from(i);
            assert_eq!(group.map(u8::from), Ok(i))
        }
    }

    #[test]
    fn try_from_invalid_i8() {
        for i in i8::MIN..0i8 {
            let language = LanguageGroup::try_from(i);
            assert_eq!(language.map(i8::from), Err(ParseLanguageGroupError))
        }

        for i in 5..=i8::MAX {
            let group = LanguageGroup::try_from(i);
            assert_eq!(group.map(i8::from), Err(ParseLanguageGroupError))
        }
    }
}