use std::fmt::{self, Display};

use redis::{FromRedisValue, RedisResult, ToRedisArgs};
use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::Serialize;
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

impl ToRedisArgs for AccountId {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        self.0.write_redis_args(out)
    }
}

impl FromRedisValue for AccountId {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        Uuid7::from_redis_value(v).map(AccountId)
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

impl Serialize for TagId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        self.0.serialize(serializer)
    }
}