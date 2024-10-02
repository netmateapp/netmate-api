use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ItemType {
    Proposal,
    Reachable,
}

impl From<ItemType> for bool {
    fn from(value: ItemType) -> Self {
        match value {
            ItemType::Proposal => true,
            ItemType::Reachable => false,
        }
    }
}

impl From<bool> for ItemType {
    fn from(value: bool) -> Self {
        if value {
            ItemType::Proposal
        } else {
            ItemType::Reachable
        }
    }
}

impl SerializeValue for ItemType {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&bool::from(*self), typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for ItemType {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        bool::from_cql(cql_val).map(ItemType::from)
    }
}