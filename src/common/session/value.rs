use std::str::FromStr;

use http::HeaderMap;
use thiserror::Error;
use time::Duration;

use crate::common::token::{calc_entropy_bytes, Token};

const SESSION_MANAGEMENT_ID_ENTROPY_BITS: usize = 120;

type SMId = Token<{calc_entropy_bytes(SESSION_MANAGEMENT_ID_ENTROPY_BITS)}>;

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

pub struct LoginSeriesId(LSId);

impl LoginSeriesId {
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



pub struct LoginId(LoginSeriesId, LoginToken);

impl LoginId {
    pub fn series_id(&self) -> &LoginSeriesId {
        &self.0
    }

    pub fn token(&self) -> &LoginToken {
        &self.1
    }
}

pub const SESSION_MANAGEMENT_COOKIE_KEY: &str = "__Host-id1";
pub const LOGIN_COOKIE_KEY: &str = "__Host-id2";

const MAX_CHROME_COOKIE_EXPIRY_DAYS: Duration = Duration::days(400);

pub struct SessionManagerImpl; // trait: Init, Fin, 

pub(crate) trait SessionManager {
    async fn initialize(&self, headers: &mut HeaderMap);
    // 生成
    // 保存
    // Set-Cookie

    async fn finalize(&self, headers: &mut HeaderMap);
    // 削除
    // Set-Cookie

    async fn reset(&self, headers: &mut HeaderMap) {
        self.delete_all_sessions().await;
        self.initialize(headers).await;
    }

    async fn delete_all_sessions(&self);

    /*
    ↓かなり複雑、ミドルウェアのロジックが出ている？
    pub async fn update() { // = 半生成 semi-generate
    // dbは使わない

    // id1クエリ
    // あれば -> ret
    // なければ -> 期限切れ処理へ
    // response -> Set-Cookie 延長

    // 期限切れ処理
    // id2クエリ -> token$account_id
    // tokenが、なければ -> ret
    // 異なれば -> on_attack() = all_delete() -> メール送信
    // 合ってれば -> id1セット & id2のトークン更新 
    // response -> Set-Cookie 延長
} */
}

/*
セキュリティのため共通化が必要な部分
・トークン生成(済)
・Cookie設定(生成、id1更新、id1&2更新)

on_gen(), on_attack() などごとにDSL定義？

handlers:
generate() or delete()
↓
middlewares:
Set-Cookie 延長 ← 前のセッションで上書きしてしまう
対策: middlewaresで下位ハンドラからのレスポンスにSet-Cookieヘッダが含まれているかチェック

delete() -> 2種削除 Set-Cookie x2, generate() -> 2種追加 Set-Cookie x2
update() -> id1延長のみ Set-Cookie x1, ローテ -> Set-Cookie x2
*/


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

fn secure_cookie_builder(key: &'static str, value: String) -> CookieBuilder<'static> {
    Cookie::build((key, value))
        .same_site(SameSite::Strict)
        .secure(true)
        .http_only(true)
        .path("/")
        .partitioned(true)
}*/