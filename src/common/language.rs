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

impl From<Language> for i8 {
  fn from(value: Language) -> Self {
      value as i8
  }
}

#[derive(Debug, PartialEq)]
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

impl TryFrom<i8> for Language {
  type Error = ParseLanguageError;

  fn try_from(value: i8) -> Result<Self, Self::Error> {
      Language::try_from(value as u8)
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
  fn try_from_valid_i8() {
    for i in 0u8..4 {
      let language = Language::try_from(i as i8);
      assert_eq!(language.map(i8::from), Ok(i as i8))
    }
  }

  #[test]
  fn try_from_invalid_i8() {
    for i in 4u8..=u8::MAX {
      let language = Language::try_from(i as i8);
      assert_eq!(language.map(i8::from), Err(ParseLanguageError))
    }
  }
}