use std::str::FromStr;

use cookie::{Cookie, SplitCookies};
use http::{header::COOKIE, HeaderMap, Request};

use crate::common::session::{cookie::{REFRESH_PAIR_COOKIE_KEY, SESSION_COOKIE_KEY}, refresh_token::RefreshToken, session_id::SessionId, session_series::SessionSeries};

pub(crate) trait ExtractSessionInformation {
    fn extract_session_information<B>(request: &Request<B>) -> (Option<SessionId>, Option<(SessionSeries, RefreshToken)>) {
        match Self::extract_cookies(request.headers()) {
            Some(cookies) => {
                let (session_id_cookie, refresh_pair_cookie) = Self::extract_session_cookies(cookies);
                (Self::parse_session_id(session_id_cookie), Self::parse_refresh_pair(refresh_pair_cookie))
            },
            None => (None, None)
        }
    }

    fn extract_cookies(headers: &HeaderMap) -> Option<SplitCookies<'_>> {
        // 攻撃を防ぐため、上限バイト数を設定する
        // __Host-id1=(11)<24>; (2)__Host-id2=(11)<24>$(1)<24> = 97bytes
        const MAX_COOKIE_BYTES: usize = 100;

        headers.get(COOKIE)
            .and_then(|v| v.to_str().ok())
            .filter(|&s| s.len() <= MAX_COOKIE_BYTES)
            .map(|s| Cookie::split_parse(s))
    }

    fn extract_session_cookies(cookies: SplitCookies<'_>) -> (Option<Cookie<'_>>, Option<Cookie<'_>>) {
        let mut session_id_cookie = None;
        let mut refresh_pair_cookie = None;
    
        // セッション管理クッキーとログインクッキーがあれば取得する
        for cookie in cookies {
            match cookie {
                Ok(cookie) => match cookie.name() {
                    SESSION_COOKIE_KEY => session_id_cookie = Some(cookie),
                    REFRESH_PAIR_COOKIE_KEY => refresh_pair_cookie = Some(cookie),
                    _ => ()
                },
                _ => ()
            }
        }
    
        (session_id_cookie, refresh_pair_cookie)
    }

    fn parse_session_id(session_id_cookie: Option<Cookie<'_>>) -> Option<SessionId> {
        session_id_cookie.and_then(|c| SessionId::from_str(c.value()).ok())
    }

    fn parse_refresh_pair(refresh_pair_cookie: Option<Cookie<'_>>) -> Option<(SessionSeries, RefreshToken)> {
        refresh_pair_cookie.and_then(|cookie| {
            let mut pair = cookie.value()
                .splitn(2, '$');

            let series_id = SessionSeries::from_str(pair.next()?)
                .ok()?;

            let token = RefreshToken::from_str(pair.next()?)
                .ok()?;
            
            Some((series_id, token))
        })
    }
}

#[cfg(test)]
mod tests {
    use http::{header::COOKIE, HeaderValue};

    use crate::common::session::{cookie::{to_cookie_value, REFRESH_PAIR_COOKIE_KEY, SESSION_COOKIE_KEY}, refresh_token::RefreshToken, session_id::SessionId, session_series::SessionSeries};

    use super::ExtractSessionInformation;

    struct MockExtractSessionIdAndRefreshPair;

    impl ExtractSessionInformation for MockExtractSessionIdAndRefreshPair {}
    
    fn test_extract_session_id_and_refresh_pair(session_id: Option<SessionId>, refresh_pair: Option<(SessionSeries, RefreshToken)>) {
        let mut cookie_header_value = String::new();

        if let Some(session_id) = &session_id {
            cookie_header_value.push_str(&format!("{}={}; ", SESSION_COOKIE_KEY, session_id.value().value()));
        }

        if let Some((series_id, token)) = &refresh_pair {
            cookie_header_value.push_str(&format!("{}={}", REFRESH_PAIR_COOKIE_KEY, to_cookie_value(&series_id, &token)));
        }

        let value = cookie_header_value
            .parse::<HeaderValue>()
            .ok();

        let request = http::Request::builder()
            .header(COOKIE, value.unwrap())
            .body(())
            .unwrap();

        let (extracted_session_id, extracted_refresh_pair) = MockExtractSessionIdAndRefreshPair::extract_session_information(&request);

        assert_eq!(session_id, extracted_session_id);
        assert_eq!(refresh_pair, extracted_refresh_pair);
    }

    #[test]
    fn session_id_and_refresh_pair() {
        test_extract_session_id_and_refresh_pair(Some(SessionId::gen()), Some((SessionSeries::gen(), RefreshToken::gen())));
    }

    #[test]
    fn session_id() {
        test_extract_session_id_and_refresh_pair(Some(SessionId::gen()), None);
    }

    #[test]
    fn refresh_pair() {
        test_extract_session_id_and_refresh_pair(None, Some((SessionSeries::gen(), RefreshToken::gen())));
    }

    #[test]
    fn none() {
        test_extract_session_id_and_refresh_pair(None, None);
    }
}