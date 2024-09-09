use std::{fmt::{self, Display, Formatter}, str::FromStr};

use cookie::{Cookie, CookieBuilder, SameSite};
use thiserror::Error;
use time::Duration;

use crate::common::token::{calc_entropy_bytes, Token};

const SESSION_ID_ENTROPY_BITS: usize = 120;

type SId = Token<{calc_entropy_bytes(SESSION_ID_ENTROPY_BITS)}>;

#[derive(Debug, PartialEq)]
pub struct SessionId(SId);

impl SessionId {
    pub fn gen() -> Self {
        Self(SId::gen())
    }

    pub fn value(&self) -> &SId {
        &self.0
    }
}

impl Display for SessionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
#[error("セッション識別子への変換に失敗しました")]
pub struct ParseSessionIdError(#[source] pub anyhow::Error);

impl FromStr for SessionId {
    type Err = ParseSessionIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Token::from_str(s)
            .map(|t| Self(t))
            .map_err(|e| ParseSessionIdError(e.into()))
    }
}

const SESSION_SERIES_ENTROPY_BITS: usize = 120;

type SS = Token<{calc_entropy_bytes(SESSION_SERIES_ENTROPY_BITS)}>;

#[derive(Debug, PartialEq)]
pub struct SessionSeries(SS);

impl SessionSeries {
    pub fn gen() -> Self {
        Self(SS::gen())
    }

    pub fn value(&self) -> &SS {
        &self.0
    }
}

impl Display for SessionSeries {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
#[error("ログイン系列識別子への変換に失敗しました")]
pub struct ParseSessionSeriesError(#[source] pub anyhow::Error);

impl FromStr for SessionSeries {
    type Err = ParseSessionSeriesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Token::from_str(s)
            .map(|t| Self(t))
            .map_err(|e| ParseSessionSeriesError(e.into()))
    }
}

const REFRESH_TOKEN_ENTROPY_BITS: usize = 120;

type RT = Token<{calc_entropy_bytes(REFRESH_TOKEN_ENTROPY_BITS)}>;

#[derive(Debug, PartialEq)]
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
            .map(|t| Self(t))
            .map_err(|e| ParseRefreshTokenError(e.into()))
    }
}


/*#[derive(Debug, PartialEq)]
pub struct LoginId(SessionSeries, RefreshToken);

impl LoginId {
    pub fn new(series_id: SessionSeries, token: RefreshToken) -> Self {
        Self(series_id, token)
    }

    pub fn series_id(&self) -> &SessionSeries {
        &self.0
    }

    pub fn token(&self) -> &RefreshToken {
        &self.1
    }
}*/

pub fn to_cookie_value(series_id: &SessionSeries, token: &RefreshToken) -> String {
    format!("{}{}{}", series_id.value().value(), REFRESH_PAIR_SEPARATOR, token.value().value())
}

pub const SESSION_COOKIE_KEY: &str = "__Host-id1";
pub const REFRESH_PAIR_COOKIE_KEY: &str = "__Host-id2";

pub const REFRESH_PAIR_SEPARATOR: char = '$';

pub const SESSION_TIMEOUT_MINUTES: Duration = Duration::minutes(30);
pub const LOGIN_ID_EXPIRY_DAYS: Duration = Duration::days(400);

// 全てのクッキーはこの関数を使用して生成されなければならない
pub fn secure_cookie_builder(key: &'static str, value: String) -> CookieBuilder<'static> {
    Cookie::build((key, value))
        .same_site(SameSite::Strict)
        .secure(true)
        .http_only(true)
        .path("/")
        .partitioned(true)
}

#[cfg(test)]
mod tests {
    use cookie::SameSite;

    use crate::common::session::value::secure_cookie_builder;

    #[test]
    fn test_secure_cookie_builder() {
        let cookie = secure_cookie_builder("key", "value".to_string()).build();
        assert_eq!(cookie.name(), "key");
        assert_eq!(cookie.value(), "value");
        assert_eq!(cookie.http_only().unwrap(), true);
        assert_eq!(cookie.secure().unwrap(), true);
        assert_eq!(cookie.same_site().unwrap(), SameSite::Strict);
        assert_eq!(cookie.path().unwrap(), "/");
        assert_eq!(cookie.partitioned().unwrap(), true);
    }
}