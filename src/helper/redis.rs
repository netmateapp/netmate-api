use bb8_redis::{bb8::{self, PooledConnection, RunError}, RedisConnectionManager};
use redis::{FromRedisValue, RedisError, ToRedisArgs};
use thiserror::Error;

pub type Pool = bb8::Pool<RedisConnectionManager>;
pub type Connection<'a> = PooledConnection<'a, RedisConnectionManager>;

const MIN_NAMESPACE_LENGTH: usize = 3;
const MAX_NAMESPACE_LENGTH: usize = 9;

#[derive(Debug)]
pub struct Namespace(&'static str);

impl Namespace {
    pub fn new(namespace: &'static str) -> Result<Self, ParseNamespaceError> {
        if namespace.contains(':') {
            Err(ParseNamespaceError::ContainsColon)
        } else if !namespace.is_ascii() {
            Err(ParseNamespaceError::NotAscii)
        } else if namespace.len() < MIN_NAMESPACE_LENGTH {
            Err(ParseNamespaceError::TooShort)
        } else if namespace.len() > MAX_NAMESPACE_LENGTH {
            Err(ParseNamespaceError::TooLong)
        } else {
            Ok(Self(namespace))
        }
    }

    pub fn value(&self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Error)]
pub enum ParseNamespaceError {
    #[error("コロンは許可されていません")]
    ContainsColon,
    #[error("ASCII文字列である必要があります")]
    NotAscii,
    #[error("{}文字以上である必要があります", MIN_NAMESPACE_LENGTH)]
    TooShort,
    #[error("{}文字以下である必要があります", MAX_NAMESPACE_LENGTH)]
    TooLong
}

pub async fn conn<O, E>(cache: &Pool, map_err: O) -> Result<Connection<'_>, E>
where
    O: FnOnce(RunError<RedisError>) -> E,
{
    cache.get()
        .await
        .map_err(|e| map_err(e))
}

pub(crate) trait TypedCommand<I, O>
where
    I: ToRedisArgs,
    O: FromRedisValue,
{
    fn query(&self, conn: &mut Connection<'_>, input: I) -> anyhow::Result<O>;
}

pub trait ToKey: ToRedisArgs {
    fn to_key(&self) -> String;

    fn write_redis_args(&self, out: &mut Vec<Vec<u8>>) {
        self.to_key().write_redis_args(out);
    }
}