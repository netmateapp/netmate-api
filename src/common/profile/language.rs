use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Language {
    Japanese = 0,
    Korean = 1,
    TaiwaneseMandarin = 2,
    AmericanEnglish = 3,
}

#[derive(Debug, PartialEq, Error)]
#[error("有効な言語ではありません")]
pub struct ParseLanguageError;

impl From<Language> for u8 {
    fn from(value: Language) -> Self {
        value as u8
    }
}

impl From<Language> for i8 {
    fn from(value: Language) -> Self {
        u8::from(value) as i8
    }
}

impl TryFrom<u8> for Language {
    type Error = ParseLanguageError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
            let language = match value {
                0 => Language::AmericanEnglish,
                1 => Language::Japanese,
                2 => Language::Korean,
                3 => Language::TaiwaneseMandarin,
                _ => return Err(ParseLanguageError)
            };
            Ok(language)
    }
}

impl TryFrom<i8> for Language {
    type Error = ParseLanguageError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        Language::try_from(value as u8)
    }
}

impl Serialize for Language {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        u8::from(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Language {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u8::deserialize(deserializer)
            .and_then(|v| Language::try_from(v).map_err(de::Error::custom))
    }
}

impl SerializeValue for Language {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&i8::from(*self), typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for Language {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        i8::from_cql(cql_val)
            .and_then(|v| Language::try_from(v).map_err(|_| FromCqlValError::BadVal))
    }
}

#[cfg(test)]
mod tests {
    use super::{Language, ParseLanguageError};

    #[test]
    fn try_from_valid_u8() {
        for i in 0u8..4 {
            let language = Language::try_from(i);
            assert_eq!(language.map(u8::from), Ok(i))
        }
    }

    #[test]
    fn try_from_invalid_u8() {
        for i in 4u8..=u8::MAX {
            let language = Language::try_from(i);
            assert_eq!(language.map(u8::from), Err(ParseLanguageError))
        }
    }

    #[test]
    fn try_from_valid_i8() {
        for i in 0u8..4 {
            let language = Language::try_from(i);
            assert_eq!(language.map(u8::from), Ok(i))
        }
    }

    #[test]
    fn try_from_invalid_i8() {
        for i in i8::MIN..0i8 {
            let language = Language::try_from(i);
            assert_eq!(language.map(i8::from), Err(ParseLanguageError))
        }

        for i in 5..=i8::MAX {
            let language = Language::try_from(i);
            assert_eq!(language.map(i8::from), Err(ParseLanguageError))
        }
    }

    #[test]
    fn deserialize_valid_json() {
        let json = r#"0"#;
        let language: Language = serde_json::from_str(json).unwrap();
        assert_eq!(language, Language::Japanese);
    }

    #[test]
    fn deserialize_invalid_json() {
        let json = r#"-1"#;
        let language = serde_json::from_str::<Language>(json);
        assert!(language.is_err());
    }
}