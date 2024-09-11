use std::fmt::Display;

use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct Uuid7(Uuid);

impl Uuid7 {
    pub fn now() -> Uuid7 {
        Uuid7(Uuid::now_v7())
    }

    pub fn value(&self) -> &Uuid {
        &self.0
    }
}

impl Display for Uuid7 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SerializeValue for Uuid7 {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        self.0.serialize(typ, writer)
    }
}

#[derive(Debug, Error)]
#[error("UUIDのバージョンが7ではありません")]
pub struct ParseUuid7Error;

impl TryFrom<Uuid> for Uuid7 {
    type Error = ParseUuid7Error;

    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        if value.get_version_num() == 7 {
            Ok(Uuid7(value))
        } else {
            Err(ParseUuid7Error)
        }
    }
}