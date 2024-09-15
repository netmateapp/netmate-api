use redis::ToRedisArgs;

pub struct SessionExpirationSeconds(u32);

impl SessionExpirationSeconds {
    pub const fn new(seconds: u32) -> Self {
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
        self.0.write_redis_args(out);
    }
}