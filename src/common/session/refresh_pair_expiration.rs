use redis::ToRedisArgs;
use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};

pub const REFRESH_PAIR_EXPIRATION: RefreshPairExpiration = RefreshPairExpiration::secs(400 * 24 * 60 * 60);

#[derive(Debug, Clone, Copy)]
pub struct RefreshPairExpiration(u32);

impl RefreshPairExpiration {
    pub const fn secs(seconds: u32) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

impl From<RefreshPairExpiration> for i32 {
    fn from(expiration: RefreshPairExpiration) -> Self {
        expiration.0 as i32
    }
}

impl SerializeValue for RefreshPairExpiration {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        (self.0 as i32).serialize(typ, writer)
    }
}

impl ToRedisArgs for RefreshPairExpiration {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        self.0.write_redis_args(out);
    }
}