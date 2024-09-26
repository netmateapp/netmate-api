use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{de, Deserialize, Deserializer};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TagRelationType {
    Inclusion,
    Equivalence,
}

impl From<TagRelationType> for bool {
    fn from(value: TagRelationType) -> Self {
        match value {
            TagRelationType::Inclusion => true,
            TagRelationType::Equivalence => false,
        }
    }
}

impl From<bool> for TagRelationType {
    fn from(value: bool) -> Self {
        if value {
            TagRelationType::Inclusion
        } else {
            TagRelationType::Equivalence
        }
    }
}

impl<'de> Deserialize<'de> for TagRelationType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        bool::deserialize(deserializer).map(TagRelationType::from)
    }
}

impl SerializeValue for TagRelationType {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&bool::from(*self), typ, writer)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Rating {
    Low,
    Middle,
    High,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum AccountTableOperationId {
    LowRating,
    MiddleRating,
    HighRating,
    Suggestion,
}

impl From<AccountTableOperationId> for u8 {
    fn from(value: AccountTableOperationId) -> Self {
        match value {
            AccountTableOperationId::LowRating => 0,
            AccountTableOperationId::MiddleRating => 1,
            AccountTableOperationId::HighRating => 2,
            AccountTableOperationId::Suggestion => 255,
        }
    }
}

impl From<AccountTableOperationId> for i8 {
    fn from(value: AccountTableOperationId) -> Self {
        u8::from(value) as i8
    }
}

impl From<Rating> for AccountTableOperationId {
    fn from(value: Rating) -> Self {
        match value {
            Rating::Low => AccountTableOperationId::LowRating,
            Rating::Middle => AccountTableOperationId::MiddleRating,
            Rating::High => AccountTableOperationId::HighRating,
        }
    }
}

impl SerializeValue for AccountTableOperationId {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&i8::from(*self), typ, writer)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CycleTableOperationId {
    LowRating,
    MiddleRating,
    HighRating,
    RemoveRating,
}

impl From<Rating> for CycleTableOperationId {
    fn from(value: Rating) -> Self {
        match value {
            Rating::Low => CycleTableOperationId::LowRating,
            Rating::Middle => CycleTableOperationId::MiddleRating,
            Rating::High => CycleTableOperationId::HighRating,
        }
    }
}

impl From<CycleTableOperationId> for u8 {
    fn from(value: CycleTableOperationId) -> Self {
        match value {
            CycleTableOperationId::LowRating => 0,
            CycleTableOperationId::MiddleRating => 1,
            CycleTableOperationId::HighRating => 2,
            CycleTableOperationId::RemoveRating => 255,
        }
    }
}

impl From<CycleTableOperationId> for i8 {
    fn from(value: CycleTableOperationId) -> Self {
        u8::from(value) as i8
    }
}

impl SerializeValue for CycleTableOperationId {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&i8::from(*self), typ, writer)
    }
}

#[cfg(test)]
mod tests {
    use crate::endpoints::tag::rating::value::{AccountTableOperationId, CycleTableOperationId, Rating, TagRelationType};

    #[test]
    fn relation_type_to_bool() {
        assert!(bool::from(TagRelationType::Inclusion));
        assert!(!bool::from(TagRelationType::Equivalence));
    }

    #[test]
    fn bool_to_tag_relation_type() {
        assert_eq!(TagRelationType::from(true), TagRelationType::Inclusion);
        assert_eq!(TagRelationType::from(false), TagRelationType::Equivalence);
    }

    #[test]
    fn u8_to_rating() {
        assert_eq!(Rating::try_from(0).unwrap(), Rating::Low);
        assert_eq!(Rating::try_from(1).unwrap(), Rating::Middle);
        assert_eq!(Rating::try_from(2).unwrap(), Rating::High);
        assert!(Rating::try_from(4).is_err());
    }

    #[test]
    fn rating_to_account_table_operation() {
        assert_eq!(AccountTableOperationId::from(Rating::Low), AccountTableOperationId::LowRating);
        assert_eq!(AccountTableOperationId::from(Rating::Middle), AccountTableOperationId::MiddleRating);
        assert_eq!(AccountTableOperationId::from(Rating::High), AccountTableOperationId::HighRating);
    }

    #[test]
    fn account_table_operation_to_u8() {
        assert_eq!(u8::from(AccountTableOperationId::LowRating), 0);
        assert_eq!(u8::from(AccountTableOperationId::MiddleRating), 1);
        assert_eq!(u8::from(AccountTableOperationId::HighRating), 2);
        assert_eq!(u8::from(AccountTableOperationId::Suggestion), 255);
    }

    #[test]
    fn rating_to_cycle_table_operation() {
        assert_eq!(CycleTableOperationId::from(Rating::Low), CycleTableOperationId::LowRating);
        assert_eq!(CycleTableOperationId::from(Rating::Middle), CycleTableOperationId::MiddleRating);
        assert_eq!(CycleTableOperationId::from(Rating::High), CycleTableOperationId::HighRating);
    }

    #[test]
    fn cycle_table_operation_to_u8() {
        assert_eq!(u8::from(CycleTableOperationId::LowRating), 0);
        assert_eq!(u8::from(CycleTableOperationId::MiddleRating), 1);
        assert_eq!(u8::from(CycleTableOperationId::HighRating), 2);
        assert_eq!(u8::from(CycleTableOperationId::RemoveRating), 255);
    }
}