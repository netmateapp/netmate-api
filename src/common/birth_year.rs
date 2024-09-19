use std::{num::NonZeroU16, sync::LazyLock, time::SystemTime};

use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// `BirthYear`は、未指定又は1900年～現在の年を表す。

pub const MIN_BIRTH_YEAR: u16 = 1900;

// 年越し時や長期稼働時に最新の年に対応できないが、
// 生年は統計目的の情報であり、数才の人間はユーザーとして想定されない
pub static MAX_BIRTH_YEAR: LazyLock<u16> = LazyLock::new(current_year);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BirthYear(Option<NonZeroU16>);

impl BirthYear {
    pub fn value(&self) -> &Option<NonZeroU16> {
        &self.0
    }
}

impl From<BirthYear> for u16 {
    fn from(value: BirthYear) -> Self {
        value.0.map_or_else(|| 0, |v| v.get())
    }
}

impl From<BirthYear> for i16 {
    fn from(value: BirthYear) -> Self {
        u16::from(value) as i16
    }
}

#[derive(Debug, PartialEq, Error)]
#[error("有効な生年ではありません")]
pub struct ParseBirthYearError;

impl TryFrom<u16> for BirthYear {
    type Error = ParseBirthYearError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value == 0 {
            Ok(BirthYear(None))
        } else if MIN_BIRTH_YEAR <= value && value <= *MAX_BIRTH_YEAR {
            Ok(BirthYear(NonZeroU16::new(value)))
        } else {
            Err(ParseBirthYearError)
        }
    }
}

impl TryFrom<i16> for BirthYear {
    type Error = ParseBirthYearError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        BirthYear::try_from(value as u16)
    }
}

impl TryFrom<Option<NonZeroU16>> for BirthYear {
    type Error = ParseBirthYearError;

    fn try_from(value: Option<NonZeroU16>) -> Result<Self, Self::Error> {
        match value {
            Some(v) if MIN_BIRTH_YEAR <= v.get() && v.get() <= *MAX_BIRTH_YEAR => Ok(BirthYear(Some(v))),
            Some(_) => Err(ParseBirthYearError),
            None => Ok(BirthYear(None))
        }
    }
}

impl Serialize for BirthYear {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.value() {
            Some(v) => serializer.serialize_some(&v.get()),
            None => serializer.serialize_none()
        }
    }
}

impl<'de> Deserialize<'de> for BirthYear {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<NonZeroU16>::deserialize(deserializer)
            .and_then(|v| BirthYear::try_from(v).map_err(de::Error::custom))
    }
}

impl SerializeValue for BirthYear {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        let value = i16::from(*self);
        SerializeValue::serialize(&value, typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for BirthYear {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        i16::from_cql(cql_val).and_then(|v| BirthYear::try_from(v).map_err(|_| FromCqlValError::BadVal))
    }
}

// unix時間から暦年を抽出する処理は、chronoの処理を利用する
// https://howardhinnant.github.io/date_algorithms.html#civil_from_days

// 年/月/日を含む3つ組をグレゴリオ暦で返します
// 前提条件: zは1970-01-01からの経過日数であり、次の範囲内である必要があります:
//           [numeric_limits<Int>::min(), numeric_limits<Int>::max() - 719468]

fn current_year() -> u16 {
    let current_time_as_secs: u64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(unixtime) => unixtime.as_secs(),
        Err(_) => panic!("システム時刻がUNIX紀元以前になっています。")
    };

    civil_year_from_unixtime(current_time_as_secs)
}

/// UNIX時間から暦年を算出する
fn civil_year_from_unixtime(unixtime_as_secs: u64) -> u16 {
    const DAYS_OFFSET: u32 = 719468;
    let z: u32 = (unixtime_as_secs / 86400) as u32 + DAYS_OFFSET; // 改変: 引数が秒であるため日数に変換
    let era = z / 146097; // 改変: zは常に0以上であるためelse節を省略
    let doe = z - era * 146097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    (y + if m <= 2 { 1 } else { 0 }) as u16
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;

    use crate::common::birth_year::{civil_year_from_unixtime, BirthYear, ParseBirthYearError, MAX_BIRTH_YEAR, MIN_BIRTH_YEAR};

    #[test]
    fn unix_epoch() {
        assert_eq!(civil_year_from_unixtime(0), 1970);
    }

    #[test]
    fn beginning_of_2000() {
        assert_eq!(civil_year_from_unixtime(946684800), 2000);
    }
    
    #[test]
    fn end_of_1999() {
        assert_eq!(civil_year_from_unixtime(946684799), 1999);
    }

    #[test]
    fn one_day_in_2024() {
        assert_eq!(civil_year_from_unixtime(1724327007), 2024);
    }

    #[test]
    fn unspecified_birth_year() {
        assert_eq!(BirthYear::try_from(0u16), Ok(BirthYear(None)));
    }

    #[test]
    fn valid_birth_year() {
        assert_eq!(BirthYear::try_from(MIN_BIRTH_YEAR), Ok(BirthYear(NonZeroU16::new(MIN_BIRTH_YEAR))));
        assert_eq!(BirthYear::try_from(*MAX_BIRTH_YEAR), Ok(BirthYear(NonZeroU16::new(*MAX_BIRTH_YEAR))));
    }

    #[test]
    fn invalid_birth_year() {
        assert_eq!(BirthYear::try_from(MIN_BIRTH_YEAR - 1), Err(ParseBirthYearError));
        assert_eq!(BirthYear::try_from(*MAX_BIRTH_YEAR + 1), Err(ParseBirthYearError));
    }

    #[test]
    fn try_from_valid_u16() {
        assert_eq!(*BirthYear::try_from(0u16).unwrap().value(), NonZeroU16::new(0));
        assert_eq!(*BirthYear::try_from(MIN_BIRTH_YEAR).unwrap().value(), NonZeroU16::new(MIN_BIRTH_YEAR));
        assert_eq!(*BirthYear::try_from(*MAX_BIRTH_YEAR).unwrap().value(), NonZeroU16::new(*MAX_BIRTH_YEAR));
    }

    #[test]
    fn try_from_invalid_u16() {
        assert_eq!(BirthYear::try_from(MIN_BIRTH_YEAR - 1), Err(ParseBirthYearError));
        assert_eq!(BirthYear::try_from(*MAX_BIRTH_YEAR + 1), Err(ParseBirthYearError));
    }

    #[test]
    fn try_from_valid_i16() {
        assert_eq!(*BirthYear::try_from(0i16).unwrap().value(), NonZeroU16::new(0));
        assert_eq!(*BirthYear::try_from(MIN_BIRTH_YEAR as i16).unwrap().value(), NonZeroU16::new(MIN_BIRTH_YEAR));
        assert_eq!(*BirthYear::try_from(*MAX_BIRTH_YEAR as i16).unwrap().value(), NonZeroU16::new(*MAX_BIRTH_YEAR));
    }

    #[test]
    fn try_from_invalid_i16() {
        assert_eq!(BirthYear::try_from((MIN_BIRTH_YEAR - 1) as i16), Err(ParseBirthYearError));
        assert_eq!(BirthYear::try_from((*MAX_BIRTH_YEAR + 1) as i16), Err(ParseBirthYearError));
    }

    #[test]
    fn try_from_valid_option_nonzero_u16() {
        assert_eq!(*BirthYear::try_from(NonZeroU16::new(0)).unwrap().value(), NonZeroU16::new(0));
        assert_eq!(*BirthYear::try_from(NonZeroU16::new(MIN_BIRTH_YEAR)).unwrap().value(), NonZeroU16::new(MIN_BIRTH_YEAR));
        assert_eq!(*BirthYear::try_from(NonZeroU16::new(*MAX_BIRTH_YEAR)).unwrap().value(), NonZeroU16::new(*MAX_BIRTH_YEAR));
    }

    #[test]
    fn try_from_invalid_option_nonzero_u16() {
        assert_eq!(BirthYear::try_from(NonZeroU16::new(MIN_BIRTH_YEAR - 1)), Err(ParseBirthYearError));
        assert_eq!(BirthYear::try_from(NonZeroU16::new(*MAX_BIRTH_YEAR + 1)), Err(ParseBirthYearError));
    }

    #[test]
    fn deserialize_valid_json() {
        let json = r#"2000"#;
        let birth_year: BirthYear = serde_json::from_str(json).unwrap();
        assert_eq!(birth_year, BirthYear::try_from(2000u16).unwrap());
    }

    #[test]
    fn deserialize_invalid_json() {
        let json = r#"1800"#;
        let birth_year = serde_json::from_str::<BirthYear>(json);
        assert!(birth_year.is_err());
    }
}
