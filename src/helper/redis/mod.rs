use bb8_redis::{bb8::{self, PooledConnection, RunError}, RedisConnectionManager};
use modname::Namespace;
use redis::RedisError;

pub type Pool = bb8::Pool<RedisConnectionManager>;

pub const SUPERTAGS_NAMESPACE: Namespace = Namespace::of("sptgs");
pub const SUBTAGS_NAMESPACE: Namespace = Namespace::of("sbtgs");

pub async fn conn<O, E>(cache: &Pool, map_err: O) -> Result<PooledConnection<'_, RedisConnectionManager>, E>
where
    O: FnOnce(RunError<RedisError>) -> E,
{
    cache.get()
        .await
        .map_err(map_err)
}

pub mod modname;