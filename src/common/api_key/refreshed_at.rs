use redis::{FromRedisValue, RedisResult, RedisWrite, ToRedisArgs};
use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::CqlValue};

use crate::common::unixtime::UnixtimeMillis;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LastApiKeyRefreshedAt(UnixtimeMillis);

impl LastApiKeyRefreshedAt {
    pub fn new(unixtime: UnixtimeMillis) -> Self {
        Self(unixtime)
    }

    pub fn value(&self) -> &UnixtimeMillis {
        &self.0
    }
}

impl ToRedisArgs for LastApiKeyRefreshedAt {
    fn write_redis_args<W: ?Sized + RedisWrite>(&self, out: &mut W) {
        UnixtimeMillis::write_redis_args(&self.value(), out);
    }
}

impl FromRedisValue for LastApiKeyRefreshedAt {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        UnixtimeMillis::from_redis_value(v).map(LastApiKeyRefreshedAt::new)
    }
}

impl FromCqlVal<Option<CqlValue>> for LastApiKeyRefreshedAt {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        UnixtimeMillis::from_cql(cql_val).map(Self::new)
    }
}
