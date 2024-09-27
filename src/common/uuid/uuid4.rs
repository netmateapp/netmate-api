use std::fmt::{self, Display};

use redis::{FromRedisValue, RedisResult, RedisWrite, ToRedisArgs};
use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Uuid4(Uuid);

impl Uuid4 {
    pub const fn new_unchecked(uuidv4: Uuid) -> Uuid4 {
        Uuid4(uuidv4)
    }

    pub fn gen() -> Uuid4 {
        Uuid4(Uuid::new_v4())
    }

    pub fn value(&self) -> Uuid {
        self.0
    }
}

impl Display for Uuid4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Error)]
#[error("UUIDのバージョンが4ではありません")]
pub struct ParseUuid4Error;

impl TryFrom<Uuid> for Uuid4 {
    type Error = ParseUuid4Error;

    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        if value.get_version_num() == 7 {
            Ok(Uuid4(value))
        } else {
            Err(ParseUuid4Error)
        }
    }
}

impl Serialize for Uuid4 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(&self.value(), serializer)
    }
}

impl<'de> Deserialize<'de> for Uuid4 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Uuid::deserialize(deserializer)
            .and_then(|v| Uuid4::try_from(v).map_err(de::Error::custom))
    }
}

impl ToRedisArgs for Uuid4 {
    fn write_redis_args<W: ?Sized + RedisWrite>(&self, out: &mut W) {
        self.value().write_redis_args(out);
    }
}

impl FromRedisValue for Uuid4 {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        Uuid::from_redis_value(v).map(Uuid4)
    }
}

impl SerializeValue for Uuid4 {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&self.value(), typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for Uuid4 {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        Uuid::from_cql(cql_val).and_then(|v| Uuid4::try_from(v).map_err(|_| FromCqlValError::BadVal))
    }
}