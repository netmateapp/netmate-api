use redis::ToRedisArgs;
use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};

pub const REFRESH_PAIR_EXPIRATION: RefreshPairExpirationSeconds = RefreshPairExpirationSeconds::secs(400 * 24 * 60 * 60);

#[derive(Debug, Clone, Copy)]
pub struct RefreshPairExpirationSeconds(u32);

impl RefreshPairExpirationSeconds {
    pub const fn secs(seconds: u32) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

impl From<RefreshPairExpirationSeconds> for i32 {
    fn from(value: RefreshPairExpirationSeconds) -> Self {
        value.as_secs() as i32
    }
}

impl SerializeValue for RefreshPairExpirationSeconds {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        i32::from(*self).serialize(typ, writer)
    }
}

impl ToRedisArgs for RefreshPairExpirationSeconds {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        self.as_secs().write_redis_args(out);
    }
}