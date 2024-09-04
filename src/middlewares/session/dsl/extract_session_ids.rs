use std::str::FromStr;

use cookie::{Cookie, SplitCookies};
use http::{header::COOKIE, HeaderMap};

use crate::common::session::value::{LoginId, LoginSeriesId, LoginToken, SessionManagementId, LOGIN_COOKIE_KEY, SESSION_MANAGEMENT_COOKIE_KEY};

pub fn extract_session_ids(request_headers: &HeaderMap) -> (Option<SessionManagementId>, Option<LoginId>) {
    match extract_cookies(request_headers) {
        Some(cookies) => convert_to_session_ids(extract_session_management_cookie_and_login_cookie(cookies)),
        None => (None, None)
    }
}

fn extract_cookies(headers: &HeaderMap) -> Option<SplitCookies<'_>> {
    // 攻撃を防ぐため、上限バイト数を決めておく
    // __Host-id1=(11)<24>; (2)__Host-id2=(11)<24>$(1)<24> = 97bytes;
    // https://developer.mozilla.org/ja/docs/Web/HTTP/Headers/Cookie
    const MAX_COOKIE_BYTES: usize = 100;

    headers.get(COOKIE)
        .and_then(|cookie_header| cookie_header.to_str().ok())
        .filter(|cookie_str| cookie_str.len() <= MAX_COOKIE_BYTES)
        .map(|cookie_str| Cookie::split_parse(cookie_str))
}

fn extract_session_management_cookie_and_login_cookie(cookies: SplitCookies<'_>) -> (Option<Cookie<'_>>, Option<Cookie<'_>>) {
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

fn convert_to_session_ids(cookies: (Option<Cookie<'_>>, Option<Cookie<'_>>)) -> (Option<SessionManagementId>, Option<LoginId>) {
    let session_management_id = cookies.0
        .map(|c| c.value().to_string())
        .map(|s| SessionManagementId::from_str(&s))
        .and_then(|r| r.ok());

    let login_id = cookies.1.and_then(|cookie| {
        let series_id_and_token = cookie.value();
        let mut parts = series_id_and_token.splitn(2, '$');
        let series_id = parts.next()?;
        let token = parts.next()?;
        
        let series_id = LoginSeriesId::from_str(series_id).ok()?;
        let token = LoginToken::from_str(token).ok()?;
        
        Some(LoginId::new(series_id, token))
    });

    (session_management_id, login_id)
}

#[cfg(test)]
mod tests {
    use http::{header::COOKIE, HeaderMap, HeaderValue};

    use crate::common::session::value::{LoginId, LoginSeriesId, LoginToken, SessionManagementId, LOGIN_COOKIE_KEY, SESSION_MANAGEMENT_COOKIE_KEY};

    use super::extract_session_ids;

    #[test]
    fn extract() {
        let session_management_id = SessionManagementId::gen();
        let login_id = LoginId::new(LoginSeriesId::gen(), LoginToken::gen());

        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(
                &format!(
                    "{}={}; {}={}${}",
                    SESSION_MANAGEMENT_COOKIE_KEY,
                    session_management_id.value().value(),
                    LOGIN_COOKIE_KEY,
                    login_id.series_id().value().value(),
                    login_id.token().value().value()
                )
            ).unwrap()
        );

        let (ex_session_management_id, ex_login_id) = extract_session_ids(&headers);
        assert_eq!(ex_session_management_id, Some(session_management_id));
        assert_eq!(ex_login_id, Some(login_id));
    }
}