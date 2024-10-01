use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{de, Deserialize, Deserializer};
use thiserror::Error;

// 各評価と数値の対応は普遍的であるため、構成要素の一部として評価を含む値と互換性がある
// テーブルの列に対応した構造体を作成する必要はなく、そのまま`Rating`を使用できる
// 評価値と対応させるため、低い評価を0、高い評価を2とする
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Rating {
    Low = 0,
    Middle = 1,
    High = 2,
}

impl From<Rating> for u8 {
    fn from(value: Rating) -> Self {
        value as u8
    }
}

impl From<Rating> for i8 {
    fn from(value: Rating) -> Self {
        u8::from(value) as i8
    }
}

#[derive(Debug, Error)]
#[error("有効な評価ではありません")]
pub struct ParseRatingError;

impl TryFrom<u8> for Rating {
    type Error = ParseRatingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let rating = match value {
            0 => Rating::Low,
            1 => Rating::Middle,
            2 => Rating::High,
            _ => return Err(ParseRatingError),
        };
        Ok(rating)
    }
}

impl<'de> Deserialize<'de> for Rating {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u8::deserialize(deserializer)
            .and_then(|v| Rating::try_from(v).map_err(de::Error::custom))
    }
}

impl SerializeValue for Rating {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&i8::from(*self), typ, writer)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::rating::Rating;

    #[test]
    fn u8_to_rating() {
        assert_eq!(Rating::try_from(0).unwrap(), Rating::Low);
        assert_eq!(Rating::try_from(1).unwrap(), Rating::Middle);
        assert_eq!(Rating::try_from(2).unwrap(), Rating::High);
        assert!(Rating::try_from(4).is_err());
    }
}