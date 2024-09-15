use redis::ToRedisArgs;

pub const SESSION_EXPIRATION: SessionExpirationSeconds = SessionExpirationSeconds::secs(30 * 60);

#[derive(Debug, Clone, Copy)]
pub struct SessionExpirationSeconds(u32);

impl SessionExpirationSeconds {
    pub const fn secs(seconds: u32) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

impl ToRedisArgs for SessionExpirationSeconds {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        self.as_secs().write_redis_args(out);
    }
}