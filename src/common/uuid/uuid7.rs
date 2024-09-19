use std::fmt::Display;

use redis::{FromRedisValue, RedisResult, RedisWrite, ToRedisArgs};
use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Uuid7(Uuid);

impl Uuid7 {
    pub const fn new_unchecked(uuid: Uuid) -> Self {
        Uuid7(uuid)
    }

    pub fn now() -> Uuid7 {
        Uuid7(Uuid::now_v7())
    }

    pub fn value(&self) -> &Uuid {
        &self.0
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

impl Display for Uuid7 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for Uuid7 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(self.value(), serializer)
    }
}

impl<'de> Deserialize<'de> for Uuid7 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Uuid::deserialize(deserializer)
            .and_then(|v| Uuid7::try_from(v).map_err(de::Error::custom))
    }
}

impl SerializeValue for Uuid7 {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(self.value(), typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for Uuid7 {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        Uuid::from_cql(cql_val).map(Uuid7)
    }
}

impl ToRedisArgs for Uuid7 {
    fn write_redis_args<W: ?Sized + RedisWrite>(&self, out: &mut W) {
        self.value().write_redis_args(out)
    }
}

impl FromRedisValue for Uuid7 {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        Uuid::from_redis_value(v).map(Uuid7)
    }
}