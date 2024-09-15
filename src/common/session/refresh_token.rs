use std::{fmt::{self, Display, Formatter}, str::FromStr};

use redis::FromRedisValue;
use thiserror::Error;

use crate::common::token::{calc_entropy_bytes, Token};

const REFRESH_TOKEN_ENTROPY_BITS: usize = 120;

type RT = Token<{calc_entropy_bytes(REFRESH_TOKEN_ENTROPY_BITS)}>;

#[derive(Debug, Clone, PartialEq)]
pub struct RefreshToken(RT);

impl RefreshToken {
    pub fn gen() -> Self {
        Self(RT::gen())
    }

    pub fn value(&self) -> &RT {
        &self.0
    }
}

impl Display for RefreshToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
#[error("ログイントークンへの変換に失敗しました")]
pub struct ParseRefreshTokenError(#[source] pub anyhow::Error);

impl FromStr for RefreshToken {
    type Err = ParseRefreshTokenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Token::from_str(s)
            .map(Self)
            .map_err(|e| ParseRefreshTokenError(e.into()))
    }
}

impl FromRedisValue for RefreshToken {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        Token::from_redis_value(v).map(Self)
    }
}