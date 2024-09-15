use cookie::{Cookie, CookieBuilder, SameSite};
use time::Duration;

use super::{refresh_token::RefreshToken, session_series::SessionSeries};

pub const SESSION_COOKIE_KEY: &str = "__Host-id1";
pub const REFRESH_PAIR_COOKIE_KEY: &str = "__Host-id2";

pub const REFRESH_PAIR_SEPARATOR: char = '$';

pub const SESSION_TIMEOUT_MINUTES: Duration = Duration::minutes(30);
pub const REFRESH_PAIR_EXPIRATION_DAYS: Duration = Duration::days(400);

pub fn to_cookie_value(series_id: &SessionSeries, token: &RefreshToken) -> String {
    format!("{}{}{}", series_id.value().value(), REFRESH_PAIR_SEPARATOR, token.value().value())
}

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

    use crate::common::session::cookie::secure_cookie_builder;

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