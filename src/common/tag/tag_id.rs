use std::fmt::{self, Display};

use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::common::uuid::uuid4::Uuid4;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TagId(Uuid4);

impl TagId {
    pub const fn of(uuid: Uuid4) -> Self {
        Self(uuid)
    }

    pub fn value(&self) -> Uuid4 {
        self.0
    }
}

impl TagId {
    pub const fn new(value: Uuid4) -> Self {
        TagId(value)
    }
}

impl Display for TagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl Serialize for TagId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(&self.value(), serializer)
    }
}

impl<'de> Deserialize<'de> for TagId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Uuid4::deserialize(deserializer).map(TagId::of)
    }
}

impl SerializeValue for TagId {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&self.value(), typ, writer)
    }
}