use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Stability {
    Stable,
    Unstable,
}

impl From<Stability> for bool {
    fn from(value: Stability) -> Self {
        match value {
            Stability::Stable => true,
            Stability::Unstable => false,
        }
    }
}

impl From<bool> for Stability {
    fn from(value: bool) -> Self {
        if value {
            Stability::Stable
        } else {
            Stability::Unstable
        }
    }
}

impl SerializeValue for Stability {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&bool::from(*self), typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for Stability {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        bool::from_cql(cql_val).map(Stability::from)
    }
}