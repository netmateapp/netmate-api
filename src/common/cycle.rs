use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};

use super::unixtime::UnixtimeMillis;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Cycle(u32);

impl Cycle {
    pub fn of(cycle: u32) -> Self {
        Self(cycle)
    }

    pub fn current_cycle() -> Self {
        const HOUR_MILLIS: u32 = 60 * 60 * 1000;
        Self(UnixtimeMillis::now().value() as u32 % HOUR_MILLIS)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl From<Cycle> for i32 {
    fn from(value: Cycle) -> Self {
        value.value() as i32
    }
}

impl From<i32> for Cycle {
    fn from(value: i32) -> Self {
        Cycle::of(value as u32)
    }
}

impl SerializeValue for Cycle {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        i32::from(*self).serialize(typ, writer)
    }
}