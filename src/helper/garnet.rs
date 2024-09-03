use bb8_redis::{bb8, RedisConnectionManager};

pub type Pool = bb8::Pool<RedisConnectionManager>;