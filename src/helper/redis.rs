use std::fmt::{self, Display};

use bb8_redis::{bb8::{self, PooledConnection, RunError}, RedisConnectionManager};
use redis::{FromRedisValue, RedisError, ToRedisArgs};

pub type Pool = bb8::Pool<RedisConnectionManager>;
pub type Connection<'a> = PooledConnection<'a, RedisConnectionManager>;

const MIN_NAMESPACE_LENGTH: usize = 3;
const MAX_NAMESPACE_LENGTH: usize = 9;

#[derive(Debug)]
pub struct Namespace(&'static str);

impl Namespace {
    pub const fn of(namespace: &'static str) -> Self {
        let bytes = namespace.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if b == b':' {
                panic!("`namespace`にコロンは使えません");
            } else if b > 0x7F {
                panic!("`namespace`はASCII文字列である必要があります");
            }
            i += 1;
        }

        if bytes.len() < MIN_NAMESPACE_LENGTH {
            panic!("`namespace`が短すぎます");
        } else if bytes.len() > MAX_NAMESPACE_LENGTH {
            panic!("`namespace`が長すぎます");
        }

        Namespace(namespace)
    }

    pub fn value(&self) -> &'static str {
        self.0
    }
}

impl Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
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