use redis::{RedisWrite, ToRedisArgs};
use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ApiKeyExpirationSeconds(u64);

impl ApiKeyExpirationSeconds {
    pub const fn secs(seconds: u64) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u64 {
        self.0
    }
}

impl From<ApiKeyExpirationSeconds> for i64 {
    fn from(expiration: ApiKeyExpirationSeconds) -> i64 {
        expiration.0 as i64
    }
}

impl ToRedisArgs for ApiKeyExpirationSeconds {
    fn write_redis_args<W: ?Sized + RedisWrite>(&self, out: &mut W) {
        u64::write_redis_args(&self.as_secs(), out);
    }
}

impl SerializeValue for ApiKeyExpirationSeconds {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        (self.0 as i64).serialize(typ, writer)
    }
}