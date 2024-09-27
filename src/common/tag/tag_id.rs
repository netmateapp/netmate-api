use std::fmt::{self, Display};

use redis::{FromRedisValue, RedisResult, RedisWrite, ToRedisArgs};
use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::common::uuid::uuid4::Uuid4;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TagId(Uuid4);

impl TagId {
    pub fn gen() -> Self {
        Self(Uuid4::gen())
    }

    pub const fn of(uuid: Uuid4) -> Self {
        Self(uuid)
    }

    pub fn value(&self) -> Uuid4 {
        self.0
    }
}

impl Display for TagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl Serialize for TagId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(&self.value(), serializer)
    }
}

impl<'de> Deserialize<'de> for TagId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Uuid4::deserialize(deserializer).map(TagId::of)
    }
}

impl ToRedisArgs for TagId {
    fn write_redis_args<W: ?Sized + RedisWrite>(&self, out: &mut W) {
        self.value().write_redis_args(out);
    }
}

impl FromRedisValue for TagId {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        Uuid4::from_redis_value(v).map(TagId::of)
    }
}

impl SerializeValue for TagId {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        SerializeValue::serialize(&self.value(), typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for TagId {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        Uuid4::from_cql(cql_val).map(TagId::of)
    }
}