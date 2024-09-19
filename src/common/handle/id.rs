use std::fmt::{self, Display};

use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::common::uuid::uuid4::Uuid4;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HandleId(Uuid4);

impl HandleId {
    pub fn gen() -> Self {
        HandleId(Uuid4::gen())
    }

    pub const fn of(value: Uuid4) -> Self {
        HandleId(value)
    }

    pub fn value(&self) -> Uuid4 {
        self.0
    }
}

impl Display for HandleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for HandleId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(&self.value(), serializer)
    }
}

impl<'de> Deserialize<'de> for HandleId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Uuid4::deserialize(deserializer)
            .map(HandleId::of)
    }
}

impl SerializeValue for HandleId {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&self.0, typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for HandleId {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        Uuid4::from_cql(cql_val).map(HandleId)
    }
}