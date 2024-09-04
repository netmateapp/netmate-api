use std::str::FromStr;

use cookie::{Cookie, SplitCookies};
use http::{header::COOKIE, HeaderMap};

use crate::common::session::value::{LoginId, LoginSeriesId, LoginToken, SessionManagementId, LOGIN_COOKIE_KEY, SESSION_MANAGEMENT_COOKIE_KEY};

pub fn extract_cookies(headers: &HeaderMap) -> Option<SplitCookies<'_>> {
    // 攻撃を防ぐため、上限バイト数を決めておく
    // __Host-id1=(11)<24>; (2)__Host-id2=(11)<24>$(1)<24> = 97bytes;
    // https://developer.mozilla.org/ja/docs/Web/HTTP/Headers/Cookie
    const MAX_COOKIE_BYTES: usize = 100;

    headers.get(COOKIE)
        .and_then(|cookie_header| cookie_header.to_str().ok())
        .filter(|cookie_str| cookie_str.len() <= MAX_COOKIE_BYTES)
        .map(|cookie_str| Cookie::split_parse(cookie_str))
}

pub fn extract_session_management_cookie_and_login_cookie(cookies: SplitCookies<'_>) -> (Option<Cookie<'_>>, Option<Cookie<'_>>) {
    let mut session_management_cookie = None;
    let mut login_cookie = None;

    // セッション管理クッキーとログインクッキーがあれば取得する
    for cookie in cookies {
        match cookie {
            Ok(cookie) => match cookie.name() {
                SESSION_MANAGEMENT_COOKIE_KEY => session_management_cookie = Some(cookie),
                LOGIN_COOKIE_KEY => login_cookie = Some(cookie),
                _ => ()
            },
            _ => ()
        }
    }

    (session_management_cookie, login_cookie)
}

pub fn convert_to_session_ids(cookies: (Option<Cookie<'_>>, Option<Cookie<'_>>)) -> (Option<SessionManagementId>, Option<LoginId>) {
    let session_management_id = cookies.0
        .map(|c| c.value().to_string())
        .map(|s| SessionManagementId::from_str(&s))
        .and_then(|r| r.ok());

    let login_id = cookies.1
        .map(|c| c.value().to_string())
        .map(|s| {
            let mut parts = s.splitn(2, '$');
            (parts.next().map(String::from), parts.next().map(String::from))
        })
        .map(|(p1, p2)| (
            p1.and_then(|p| LoginSeriesId::from_str(p.as_str()).ok()),
            p2.and_then(|p| LoginToken::from_str(p.as_str()).ok())
        ))
        .and_then(|(series_id, token)| {
            series_id.and_then(|series| token.map(|tok| LoginId::new(series, tok)))
        });
    
    (session_management_id, login_id)
}