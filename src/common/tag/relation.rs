use std::fmt::{self, Display};

use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{Deserialize, Deserializer};
use thiserror::Error;

use super::non_top_tag::NonTopTagId;

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

impl Display for TagRelation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TagRelation::Inclusion => write!(f, "包含"),
            TagRelation::Equivalence => write!(f, "同値")
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

pub fn validate_tag_relation(subtag_id: NonTopTagId, supertag_id: NonTopTagId, relation: TagRelation) -> Result<(), TagRelationError> {
    if subtag_id == supertag_id {
        Err(TagRelationError::CannotRateSameTagRelation)
    } else if relation == TagRelation::Equivalence && subtag_id > supertag_id {
        Err(TagRelationError::SubtagIdMustBeSmallerThanSupertagIdInEquivalence)
    } else {
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum TagRelationError {
    #[error("同じタグ間の関係を評価することはできません")]
    CannotRateSameTagRelation,
    #[error("同値関係では`subtag_id`が`supertag_id`より小さくなければなりません")]
    SubtagIdMustBeSmallerThanSupertagIdInEquivalence,
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::common::{tag::{non_top_tag::NonTopTagId, relation::TagRelation, tag_id::TagId}, uuid::uuid4::Uuid4};

    use super::{validate_tag_relation, TagRelationError};

    #[test]
    fn same_tag() {
        let tag_id = NonTopTagId::gen();

        for relation in [TagRelation::Inclusion, TagRelation::Equivalence] {
            assert!(matches!(validate_tag_relation(tag_id, tag_id, relation).err().unwrap(), TagRelationError::CannotRateSameTagRelation));
        }
    }

    #[test]
    fn compare_tags_in_equivalence_relation() {
        let subtag_id = NonTopTagId::try_from(TagId::of(Uuid4::new_unchecked(Uuid::from_fields(0x01, 0x01, 0x4001, &[0x80, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01])))).unwrap();
        let supertag_id = NonTopTagId::try_from(TagId::of(Uuid4::new_unchecked(Uuid::from_fields(0x01, 0x01, 0x4001, &[0x80, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02])))).unwrap();

        assert!(validate_tag_relation(subtag_id, supertag_id, TagRelation::Equivalence).is_ok());

        // 下位タグと上位タグを逆転させる
        assert!(matches!(validate_tag_relation(supertag_id, subtag_id, TagRelation::Equivalence).err().unwrap(), TagRelationError::SubtagIdMustBeSmallerThanSupertagIdInEquivalence));
    }

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