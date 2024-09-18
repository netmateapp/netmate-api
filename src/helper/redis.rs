use std::fmt::{self, Display};

use bb8_redis::{bb8::{self, PooledConnection, RunError}, RedisConnectionManager};
use redis::RedisError;
use thiserror::Error;

pub type Pool = bb8::Pool<RedisConnectionManager>;

pub async fn conn<O, E>(cache: &Pool, map_err: O) -> Result<PooledConnection<'_, RedisConnectionManager>, E>
where
    O: FnOnce(RunError<RedisError>) -> E,
{
    cache.get()
        .await
        .map_err(map_err)
}

pub const NAMESPACE_SEPARATOR: char = ':';

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

impl Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
