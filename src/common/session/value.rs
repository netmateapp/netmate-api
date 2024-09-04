use std::str::FromStr;

use cookie::{Cookie, CookieBuilder, SameSite};
use http::HeaderMap;
use thiserror::Error;
use time::Duration;

use crate::common::token::{calc_entropy_bytes, Token};

const SESSION_MANAGEMENT_ID_ENTROPY_BITS: usize = 120;

type SMId = Token<{calc_entropy_bytes(SESSION_MANAGEMENT_ID_ENTROPY_BITS)}>;

#[derive(Debug, PartialEq)]
pub struct SessionManagementId(SMId);

impl SessionManagementId {
    pub fn gen() -> Self {
        Self(SMId::gen())
    }

    pub fn value(&self) -> &SMId {
        &self.0
    }
}

#[derive(Debug, Error)]
#[error("セッション管理識別子への変換に失敗しました")]
pub struct ParseSessionManagementIdError(#[source] pub anyhow::Error);

impl FromStr for SessionManagementId {
    type Err = ParseSessionManagementIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Token::from_str(s)
            .map(|t| Self(t))
            .map_err(|e| ParseSessionManagementIdError(e.into()))
    }
}



const LOGIN_COOKIE_SERIES_ID_ENTROPY_BITS: usize = 120;

type LSId = Token<{calc_entropy_bytes(LOGIN_COOKIE_SERIES_ID_ENTROPY_BITS)}>;

#[derive(Debug, PartialEq)]
pub struct LoginSeriesId(LSId);

impl LoginSeriesId {
    pub fn gen() -> Self {
        Self(LSId::gen())
    }

    pub fn value(&self) -> &LSId {
        &self.0
    }
}

#[derive(Debug, Error)]
#[error("ログイン系列識別子への変換に失敗しました")]
pub struct ParseLoginSeriesIdError(#[source] pub anyhow::Error);

impl FromStr for LoginSeriesId {
    type Err = ParseLoginSeriesIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Token::from_str(s)
            .map(|t| Self(t))
            .map_err(|e| ParseLoginSeriesIdError(e.into()))
    }
}



const LOGIN_COOKIE_TOKEN_ENTROPY_BITS: usize = 120;

type LT = Token<{calc_entropy_bytes(LOGIN_COOKIE_TOKEN_ENTROPY_BITS)}>;

#[derive(Debug, PartialEq)]
pub struct LoginToken(LT);

impl LoginToken {
    pub fn gen() -> Self {
        Self(LT::gen())
    }

    pub fn value(&self) -> &LT {
        &self.0
    }
}

#[derive(Debug, Error)]
#[error("ログイントークンへの変換に失敗しました")]
pub struct ParseLoginTokenError(#[source] pub anyhow::Error);

impl FromStr for LoginToken {
    type Err = ParseLoginTokenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Token::from_str(s)
            .map(|t| Self(t))
            .map_err(|e| ParseLoginTokenError(e.into()))
    }
}


#[derive(Debug, PartialEq)]
pub struct LoginId(LoginSeriesId, LoginToken);

impl LoginId {
    pub fn new(series_id: LoginSeriesId, token: LoginToken) -> Self {
        Self(series_id, token)
    }

    pub fn series_id(&self) -> &LoginSeriesId {
        &self.0
    }

    pub fn token(&self) -> &LoginToken {
        &self.1
    }
}

pub fn to_cookie_value(series_id: &LoginSeriesId, token: &LoginToken) -> String {
    format!("{}:{}", series_id.value().value(), token.value().value())
}

pub const SESSION_MANAGEMENT_COOKIE_KEY: &str = "__Host-id1";
pub const LOGIN_COOKIE_KEY: &str = "__Host-id2";

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

/*
fn append() {
    set_cookie(response.headers_mut(), &gen_session_management_cookie());
    set_cookie(response.headers_mut(), &gen_login_cookie());
}

fn set_cookie(headers: &mut HeaderMap, cookie: &Cookie<'static>) {
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(cookie.to_string().as_str()).unwrap()
    );
}

fn gen_session_management_cookie() -> Cookie<'static> {
    secure_cookie_builder(
        SESSION_MANAGEMENT_COOKIE_KEY,
        SessionManagementId::gen().value().clone()
    ).build()
}

fn gen_login_cookie() -> Cookie<'static> {
    login_cookie(LoginSeriesId::gen(), LoginToken::gen())
}

fn login_cookie_with_new_token(series_id: LoginSeriesId) -> Cookie<'static> {
    login_cookie(series_id, LoginToken::gen())
}

fn login_cookie(series_id: LoginSeriesId, token: LoginToken) -> Cookie<'static> {
    secure_cookie_builder(LOGIN_COOKIE_KEY, LoginId::new(series_id, token).0)
        .max_age(MAX_CHROME_COOKIE_EXPIRY_DAYS)
        .build()
}

*/