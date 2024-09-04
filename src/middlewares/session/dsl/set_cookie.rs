use cookie::Cookie;
use http::{header::SET_COOKIE, HeaderMap, HeaderName, HeaderValue};
use time::Duration;

use crate::common::session::value::{secure_cookie_builder, to_cookie_value, LoginSeriesId, LoginToken, SessionManagementId, LOGIN_COOKIE_KEY, LOGIN_ID_EXPIRY_DAYS, SESSION_MANAGEMENT_COOKIE_KEY, SESSION_TIMEOUT_MINUTES};

// 期限の延長はクッキーの再設定により行われるため、実態はセッション識別子の再設定関数である
pub fn reset_session_timeout(response_headers: &mut HeaderMap, session_management_id: &SessionManagementId) {
    set_new_session_management_id_in_header(response_headers, session_management_id);
}

pub fn set_new_session_management_id_in_header(response_headers: &mut HeaderMap, new_session_management_id: &SessionManagementId) {
    let cookie = secure_cookie_builder(&SESSION_MANAGEMENT_COOKIE_KEY, new_session_management_id.value().value().clone())
        .max_age(SESSION_TIMEOUT_MINUTES)
        .build();

    set_cookie(response_headers, &cookie);
}

pub fn set_new_login_token_in_header(response_headers: &mut HeaderMap, series_id: &LoginSeriesId, new_login_token: &LoginToken) {
    let cookie = secure_cookie_builder(&LOGIN_COOKIE_KEY, to_cookie_value(series_id, new_login_token))
        .max_age(LOGIN_ID_EXPIRY_DAYS)
        .build();

    set_cookie(response_headers, &cookie);
}

pub fn clear_session_ids_in_response_header() -> HeaderMap {
    let mut headers = HeaderMap::new();

    let session_management_cookie = secure_cookie_builder(&SESSION_MANAGEMENT_COOKIE_KEY, String::from(""))
        .max_age(Duration::seconds(0))
        .build();
    let login_cookie = secure_cookie_builder(&LOGIN_COOKIE_KEY, String::from(""))
        .max_age(Duration::seconds(0))
        .build();

    set_cookie(&mut headers, &session_management_cookie);
    set_cookie(&mut headers, &login_cookie);

    headers
}

fn set_cookie(headers: &mut HeaderMap, cookie: &Cookie<'static>) {
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(cookie.to_string().as_str()).unwrap()
    );
}
