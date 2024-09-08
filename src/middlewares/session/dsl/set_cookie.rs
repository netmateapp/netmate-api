use cookie::Cookie;
use http::{header::SET_COOKIE, HeaderMap, HeaderName, HeaderValue, Response};
use time::Duration;

use crate::common::session::value::{secure_cookie_builder, to_cookie_value, SessionSeries, RefreshToken, SessionId, REFRESH_PAIR_COOKIE_KEY, LOGIN_ID_EXPIRY_DAYS, SESSION_COOKIE_KEY, SESSION_TIMEOUT_MINUTES};

pub(crate) trait SetSessionCookie {
    fn refresh_session_cookie_expiry<B>(response: &mut Response<B>, session_id: &SessionId) {
        Self::set_session_cookie_with_expiry(response, session_id);
    }

    fn set_session_cookie_with_expiry<B>(response: &mut Response<B>, session_id: &SessionId) {
        Self::set_cookie(response, &SESSION_COOKIE_KEY, String::from(session_id.value().value()), SESSION_TIMEOUT_MINUTES)
    }

    fn set_refresh_pair_cookie_with_expiry<B>(response: &mut Response<B>, session_series: &SessionSeries, refresh_token: &RefreshToken) {
        Self::set_cookie(response, &REFRESH_PAIR_COOKIE_KEY, to_cookie_value(session_series, refresh_token), LOGIN_ID_EXPIRY_DAYS)
    }

    fn set_cookie<B>(response: &mut Response<B>, key: &'static str, value: String, max_age: Duration) {
        let cookie = secure_cookie_builder(key, value)
            .max_age(max_age)
            .build();

        // Cookieのキー及び値に無効な文字を使用していないため、`unwrap()`で問題ない
        let header_value = HeaderValue::from_str(cookie.to_string().as_str()).unwrap();

        response.headers_mut().insert(SET_COOKIE, header_value);
    }

    fn clear_session_cookies(&self) -> [(HeaderName, HeaderValue); 2];
}


// 期限の延長はクッキーの再設定により行われるため、実態はセッション識別子の再設定関数である
pub fn reset_session_timeout(response_headers: &mut HeaderMap, session_management_id: &SessionId) {
    set_new_session_id_into_response_header(response_headers, session_management_id);
}

pub fn set_new_session_id_into_response_header(response_headers: &mut HeaderMap, new_session_management_id: &SessionId) {
    let cookie = secure_cookie_builder(&SESSION_COOKIE_KEY, new_session_management_id.value().value().clone())
        .max_age(SESSION_TIMEOUT_MINUTES)
        .build();

    set_cookie(response_headers, &cookie);
}

pub fn set_new_refresh_token_in_header(response_headers: &mut HeaderMap, series_id: &SessionSeries, refresh_token: &RefreshToken) {
    let cookie = secure_cookie_builder(&REFRESH_PAIR_COOKIE_KEY, to_cookie_value(series_id, refresh_token))
        .max_age(LOGIN_ID_EXPIRY_DAYS)
        .build();

    set_cookie(response_headers, &cookie);
}

fn set_cookie(headers: &mut HeaderMap, cookie: &Cookie<'static>) {
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(cookie.to_string().as_str()).unwrap()
    );
}

pub fn clear_session_ids_in_response_header() -> [(HeaderName, HeaderValue); 2] {
    let session_management_cookie = secure_cookie_builder(&SESSION_COOKIE_KEY, String::from(""))
        .max_age(Duration::seconds(0))
        .build();

    let login_cookie = secure_cookie_builder(&REFRESH_PAIR_COOKIE_KEY, String::from(""));

    [
        (SET_COOKIE, HeaderValue::from_str(session_management_cookie.to_string().as_str()).unwrap()),
        (SET_COOKIE, HeaderValue::from_str(login_cookie.to_string().as_str()).unwrap())
    ]
}


#[cfg(test)]
mod tests {
    use cookie::Cookie;
    use http::{header::SET_COOKIE, HeaderMap};

    use crate::{common::session::value::{to_cookie_value, SessionSeries, RefreshToken, SessionId, REFRESH_PAIR_COOKIE_KEY, LOGIN_ID_EXPIRY_DAYS, SESSION_COOKIE_KEY, SESSION_TIMEOUT_MINUTES}, middlewares::session::dsl::set_cookie::{set_new_refresh_token_in_header, set_new_session_id_into_response_header}};

    use super::set_cookie;

    #[test]
    fn test_set_new_session_management_id_in_header() {
        let mut headers = HeaderMap::new();
        let id = SessionId::gen();

        set_new_session_id_into_response_header(&mut headers, &id);

        let cookie = parse_cookie(&headers);

        assert_eq!(cookie.name(), SESSION_COOKIE_KEY);
        assert_eq!(cookie.value(), id.value().value().as_str());
        assert_eq!(cookie.max_age().unwrap(), SESSION_TIMEOUT_MINUTES);
    }

    #[test]
    fn test_set_new_login_token_in_header() {
        let mut headers = HeaderMap::new();
        let series_id = SessionSeries::gen();
        let token = RefreshToken::gen();

        set_new_refresh_token_in_header(&mut headers, &series_id, &token);

        let cookie = parse_cookie(&headers);

        assert_eq!(cookie.name(), REFRESH_PAIR_COOKIE_KEY);
        assert_eq!(cookie.value(), to_cookie_value(&series_id, &token));
        assert_eq!(cookie.max_age().unwrap(), LOGIN_ID_EXPIRY_DAYS);
    }

    #[test]
    fn test_set_cookie() {
        let mut headers = HeaderMap::new();
        let cookie = Cookie::new("key", "value");

        set_cookie(&mut headers, &cookie);

        let cookie = parse_cookie(&headers);

        assert_eq!(cookie.name(), "key");
        assert_eq!(cookie.value(), "value");
    }

    fn parse_cookie<'a>(headers: &'a HeaderMap) -> Cookie<'a> {
        let cookie_str = headers.get(SET_COOKIE).unwrap().to_str().unwrap();
        Cookie::parse(cookie_str).unwrap()
    }
}