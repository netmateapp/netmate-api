use std::time::{SystemTime, UNIX_EPOCH};

use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::{response::result::{ColumnType, CqlValue}, value::CqlTimestamp}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct UnixtimeMillis(u64);

impl UnixtimeMillis {
    pub fn of(unixtime_millis: u64) -> Self {
        Self(unixtime_millis)
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
        i64::from(*self).serialize(typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for UnixtimeMillis {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        CqlTimestamp::from_cql(cql_val)
            .map(|cql_timestamp| UnixtimeMillis::from(cql_timestamp.0))
    }
}