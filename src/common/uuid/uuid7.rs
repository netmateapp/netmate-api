use std::fmt::Display;

use redis::{FromRedisValue, RedisResult, ToRedisArgs};
use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
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

impl FromCqlVal<Option<CqlValue>> for Uuid7 {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        Uuid::from_cql(cql_val).map(Uuid7)
    }
}

impl ToRedisArgs for Uuid7 {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        self.0.write_redis_args(out)
    }
}

impl FromRedisValue for Uuid7 {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        Uuid::from_redis_value(v).map(Uuid7)
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