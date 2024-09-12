use std::time::{SystemTime, UNIX_EPOCH};

use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};

pub struct UnixtimeMillis(u64);

impl UnixtimeMillis {
    pub fn new(unixtime: u64) -> Self {
        Self(unixtime)
    }

    pub fn now() -> Self {
        // プログラム開始時に時刻の正常性を確認しているため、`unwrap()`で問題ない
        Self(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl From<i64> for UnixtimeMillis {
    fn from(unixtime: i64) -> Self {
        Self(unixtime as u64)
    }
}

impl From<UnixtimeMillis> for i64 {
    fn from(unixtime: UnixtimeMillis) -> Self {
        unixtime.0 as i64
    }
}

impl SerializeValue for UnixtimeMillis {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        (self.0 as i64).serialize(typ, writer)
    }
}