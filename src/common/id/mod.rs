use std::fmt::{self, Display};

use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use uuid4::Uuid4;
use uuid7::Uuid7;

pub mod uuid4;
pub mod uuid7;

#[derive(Debug, Clone, PartialEq)]
pub struct AccountId(Uuid7);

impl AccountId {
    pub fn gen() -> Self {
        AccountId(Uuid7::now())
    }

    pub const fn of(value: Uuid7) -> Self {
        AccountId(value)
    }

    pub fn value(&self) -> &Uuid7 {
        &self.0
    }
}

impl Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SerializeValue for AccountId {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        self.0.serialize(typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for AccountId {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        Uuid7::from_cql(cql_val).map(AccountId)
    }
}

pub struct TagId(Uuid4);

impl TagId {
    pub fn value(&self) -> &Uuid4 {
        &self.0
    }
}

impl TagId {
    pub const fn new(value: Uuid4) -> Self {
        TagId(value)
    }
}