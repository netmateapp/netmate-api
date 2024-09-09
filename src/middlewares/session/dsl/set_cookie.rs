use http::{header::SET_COOKIE, HeaderName, HeaderValue, Response};
use time::Duration;

use crate::common::session::value::{secure_cookie_builder, to_cookie_value, SessionSeries, RefreshToken, SessionId, REFRESH_PAIR_COOKIE_KEY, REFRESH_PAIR_EXPIRATION_DAYS, SESSION_COOKIE_KEY, SESSION_TIMEOUT_MINUTES};

pub(crate) trait SetSessionCookie {
    fn refresh_session_cookie_expiration<B>(response: &mut Response<B>, session_id: &SessionId) {
        Self::set_session_cookie_with_expiration(response, session_id);
    }

    fn set_session_cookie_with_expiration<B>(response: &mut Response<B>, session_id: &SessionId) {
        Self::set_cookie(response, &SESSION_COOKIE_KEY, String::from(session_id.value().value()), SESSION_TIMEOUT_MINUTES)
    }

    fn set_refresh_pair_cookie_with_expiration<B>(response: &mut Response<B>, session_series: &SessionSeries, refresh_token: &RefreshToken) {
        Self::set_cookie(response, &REFRESH_PAIR_COOKIE_KEY, to_cookie_value(session_series, refresh_token), REFRESH_PAIR_EXPIRATION_DAYS)
    }

    fn clear_session_related_cookie_headers() -> [(HeaderName, HeaderValue); 2] {
        [
            (SET_COOKIE, Self::create_cookie_value(&SESSION_COOKIE_KEY, String::from(""), Duration::seconds(0))),
            (SET_COOKIE, Self::create_cookie_value(&REFRESH_PAIR_COOKIE_KEY, String::from(""), Duration::seconds(0)))
        ]
    }

    fn set_cookie<B>(response: &mut Response<B>, key: &'static str, value: String, max_age: Duration) {
        response.headers_mut().insert(SET_COOKIE, Self::create_cookie_value(key, value, max_age));
    }

    fn create_cookie_value(key: &'static str, value: String, max_age: Duration) -> HeaderValue {
        let cookie = secure_cookie_builder(key, value)
            .max_age(max_age)
            .build();

        // Cookieのキー及び値に無効な文字を使用していないため、`unwrap()`で問題ない
        HeaderValue::from_str(cookie.to_string().as_str()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use cookie::Cookie;
    use http::{header::SET_COOKIE, Response};
    use time::Duration;

    use crate::common::session::value::{to_cookie_value, RefreshToken, SessionId, SessionSeries, REFRESH_PAIR_COOKIE_KEY, REFRESH_PAIR_EXPIRATION_DAYS, SESSION_COOKIE_KEY, SESSION_TIMEOUT_MINUTES};

    use super::SetSessionCookie;

    struct MockSetSessionCookie;

    impl SetSessionCookie for MockSetSessionCookie {}

    fn test_set_cookie(response: Response<()>, key: &'static str, value: String, max_age: Duration) {
        let set_cookie_value = response.headers()
            .get(SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap();

        let cookie = Cookie::parse(set_cookie_value)
            .unwrap();

        assert_eq!(cookie.name(), key);
        assert_eq!(cookie.value(), value);
        assert_eq!(cookie.max_age().unwrap(), max_age);
    }

    #[test]
    fn set_session_cookie() {
        let mut response = Response::new(());
        let session_id = SessionId::gen();
        MockSetSessionCookie::set_session_cookie_with_expiration(&mut response, &session_id);
        test_set_cookie(response, &SESSION_COOKIE_KEY, session_id.to_string(), SESSION_TIMEOUT_MINUTES);
    }

    #[test]
    fn set_refresh_pair() {
        let mut response = Response::new(());
        let session_series = SessionSeries::gen();
        let refresh_token = RefreshToken::gen();
        MockSetSessionCookie::set_refresh_pair_cookie_with_expiration(&mut response, &session_series, &refresh_token);
        test_set_cookie(response, &REFRESH_PAIR_COOKIE_KEY, to_cookie_value(&session_series, &refresh_token), REFRESH_PAIR_EXPIRATION_DAYS);
    }

    #[test]
    fn set_cookie() {
        let mut response = Response::new(());
        MockSetSessionCookie::set_cookie(&mut response, "key", String::from("value"), Duration::seconds(10));
        test_set_cookie(response, "key", String::from("value"), Duration::seconds(10));
    }
}