use redis::ToRedisArgs;
use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};

#[derive(Debug, Clone, Copy)]
pub struct RefreshPairExpirationSeconds(u32);

impl RefreshPairExpirationSeconds {
    pub const fn new(seconds: u32) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

impl From<RefreshPairExpirationSeconds> for i32 {
    fn from(expiration: RefreshPairExpirationSeconds) -> Self {
        expiration.0 as i32
    }
}

impl SerializeValue for RefreshPairExpirationSeconds {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        (self.0 as i32).serialize(typ, writer)
    }
}

impl ToRedisArgs for RefreshPairExpirationSeconds {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        self.0.write_redis_args(out);
    }
}