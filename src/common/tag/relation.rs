use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{Deserialize, Deserializer};

// タグ関係は包含関係であり、あるタグ間に包含関係が存在する場合、
// 一方が他方に包含関係を持つか、両者が相互に包含関係を持つかのどちらかである
// そのためboolで表現できる
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TagRelation {
    Inclusion,
    Equivalence,
}

impl From<TagRelation> for bool {
    fn from(value: TagRelation) -> Self {
        match value {
            TagRelation::Inclusion => true,
            TagRelation::Equivalence => false,
        }
    }
}

impl From<bool> for TagRelation {
    fn from(value: bool) -> Self {
        if value {
            TagRelation::Inclusion
        } else {
            TagRelation::Equivalence
        }
    }
}

impl<'de> Deserialize<'de> for TagRelation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        bool::deserialize(deserializer).map(TagRelation::from)
    }
}

impl SerializeValue for TagRelation {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&bool::from(*self), typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for TagRelation {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        bool::from_cql(cql_val).map(TagRelation::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::tag::relation::TagRelation;

    #[test]
    fn relation_type_to_bool() {
        assert!(bool::from(TagRelation::Inclusion));
        assert!(!bool::from(TagRelation::Equivalence));
    }

    #[test]
    fn bool_to_tag_relation_type() {
        assert_eq!(TagRelation::from(true), TagRelation::Inclusion);
        assert_eq!(TagRelation::from(false), TagRelation::Equivalence);
    }
}