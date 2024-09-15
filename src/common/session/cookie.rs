use axum::response::Response;
use cookie::{Cookie, CookieBuilder, SameSite};
use http::{header::SET_COOKIE, HeaderValue};
use time::Duration;

use super::{refresh_token::RefreshToken, session_id::SessionId, session_series::SessionSeries};

pub const SESSION_COOKIE_KEY: &str = "__Host-id1";
pub const REFRESH_PAIR_COOKIE_KEY: &str = "__Host-id2";

pub const REFRESH_PAIR_SEPARATOR: char = '$';

pub const SESSION_TIMEOUT_MINUTES: Duration = Duration::minutes(30);
pub const REFRESH_PAIR_EXPIRATION_DAYS: Duration = Duration::days(400);

fn refresh_session_cookie_expiration<B>(response: &mut Response<B>, session_id: &SessionId) {
    set_session_cookie_with_expiration(response, session_id);
}

pub fn set_session_cookie_with_expiration<B>(response: &mut Response<B>, session_id: &SessionId) {
    set_cookie(response, &SESSION_COOKIE_KEY, String::from(session_id.value().value()), SESSION_TIMEOUT_MINUTES)
}

pub fn set_refresh_pair_cookie_with_expiration<B>(response: &mut Response<B>, session_series: &SessionSeries, refresh_token: &RefreshToken) {
    set_cookie(response, &REFRESH_PAIR_COOKIE_KEY, to_cookie_value(session_series, refresh_token), REFRESH_PAIR_EXPIRATION_DAYS)
}

fn set_cookie<B>(response: &mut Response<B>, key: &'static str, value: String, max_age: Duration) {
    response.headers_mut().insert(SET_COOKIE, create_cookie_value(key, value, max_age));
}

fn create_cookie_value(key: &'static str, value: String, max_age: Duration) -> HeaderValue {
    let cookie = secure_cookie_builder(key, value)
        .max_age(max_age)
        .build();

    // Cookieのキー及び値に無効な文字を使用していないため、`unwrap()`で問題ない
    HeaderValue::from_str(cookie.to_string().as_str()).unwrap()
}

pub fn to_cookie_value(series_id: &SessionSeries, token: &RefreshToken) -> String {
    format!("{}{}{}", series_id.value().value(), REFRESH_PAIR_SEPARATOR, token.value().value())
}

// 全てのクッキーはこの関数を使用して生成されなければならない
fn secure_cookie_builder(key: &'static str, value: String) -> CookieBuilder<'static> {
    Cookie::build((key, value))
        .same_site(SameSite::Strict)
        .secure(true)
        .http_only(true)
        .path("/")
        .partitioned(true)
}

#[cfg(test)]
mod tests {
    use cookie::{Cookie, SameSite};
    use http::{header::SET_COOKIE, Response};
    use time::Duration;

    use crate::common::session::{cookie::secure_cookie_builder, refresh_token::RefreshToken, session_id::SessionId, session_series::SessionSeries};

    use super::{set_cookie, set_refresh_pair_cookie_with_expiration, set_session_cookie_with_expiration, to_cookie_value, REFRESH_PAIR_COOKIE_KEY, REFRESH_PAIR_EXPIRATION_DAYS, SESSION_COOKIE_KEY, SESSION_TIMEOUT_MINUTES};

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
        set_session_cookie_with_expiration(&mut response, &session_id);
        test_set_cookie(response, &SESSION_COOKIE_KEY, session_id.to_string(), SESSION_TIMEOUT_MINUTES);
    }

    #[test]
    fn set_refresh_pair() {
        let mut response = Response::new(());
        let session_series = SessionSeries::gen();
        let refresh_token = RefreshToken::gen();
        set_refresh_pair_cookie_with_expiration(&mut response, &session_series, &refresh_token);
        test_set_cookie(response, &REFRESH_PAIR_COOKIE_KEY, to_cookie_value(&session_series, &refresh_token), REFRESH_PAIR_EXPIRATION_DAYS);
    }

    #[test]
    fn set() {
        let mut response = Response::new(());
        set_cookie(&mut response, "key", String::from("value"), Duration::seconds(10));
        test_set_cookie(response, "key", String::from("value"), Duration::seconds(10));
    }

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