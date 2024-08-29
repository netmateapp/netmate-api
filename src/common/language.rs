use serde::{de, Deserialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
  AmericanEnglish = 0,
  Japanese = 1,
  Korean = 2,
  TaiwaneseMandarin = 3,
}

impl From<Language> for u8 {
  fn from(value: Language) -> Self {
      value as u8
  }
}

#[derive(Debug, PartialEq, Error)]
#[error("有効な言語ではありません")]
pub struct ParseLanguageError;

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

impl<'de> Deserialize<'de> for Language {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
      D: serde::Deserializer<'de>
  {
      let n: u8 = Deserialize::deserialize(deserializer)?;
      Language::try_from(n).map_err(de::Error::custom)
  }
}

#[cfg(test)]
mod tests {
    use crate::common::language::ParseLanguageError;

    use super::Language;

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
  fn deserialize_valid_json() {
    let json = r#"0"#;
    let language: Language = serde_json::from_str(json).unwrap();
    assert_eq!(language, Language::AmericanEnglish);
  }

  #[test]
  fn deserialize_invalid_json() {
    let json = r#"-1"#;
    let language = serde_json::from_str::<Language>(json);
    assert!(language.is_err());
  }
}