use redis::ToRedisArgs;

pub const SESSION_EXPIRATION: SessionExpiration = SessionExpiration::secs(30 * 60);

#[derive(Debug, Clone, Copy)]
pub struct SessionExpiration(u32);

impl SessionExpiration {
    pub const fn secs(seconds: u32) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

impl ToRedisArgs for SessionExpiration {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        self.0.write_redis_args(out);
    }
}