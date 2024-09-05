use std::time::{SystemTime, UNIX_EPOCH};

use http::{header::SET_COOKIE, Extensions, HeaderMap};

use crate::{common::{fallible::Fallible, id::AccountId, session::value::LoginToken, unixtime::UnixtimeMillis}, middlewares::session::dsl::ManageSessionError};

pub fn insert_account_id(extensions: &mut Extensions, account_id: AccountId) {
    extensions.insert(account_id);
}

pub fn can_set_cookie_in_response_header(headers: &HeaderMap) -> bool {
    !headers.contains_key(SET_COOKIE)
}

pub fn is_same_token(request_token: &LoginToken, registered_token: &LoginToken) -> bool {
    request_token.value().value() == registered_token.value().value()
}

const SESSION_EXTENSION_THRESHOLD: u64 = 30 * 24 * 60 * 60 * 1000;

pub struct SeriesIdRefreshTimestamp(UnixtimeMillis);

impl SeriesIdRefreshTimestamp {
    pub fn new(unixtime: UnixtimeMillis) -> Self {
        Self(unixtime)
    }

    pub fn value(&self) -> &UnixtimeMillis {
        &self.0
    }
}

pub fn should_extend_series_id_expiration(timestamp: &SeriesIdRefreshTimestamp) -> Fallible<bool, ManageSessionError> {
    let current_unixtime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .map_err(|e| ManageSessionError::CheckSeriesIdExpirationExtendabilityFailed(e.into()))?;

    Ok(current_unixtime - timestamp.value().value() > SESSION_EXTENSION_THRESHOLD)
}

#[cfg(test)]
mod tests {
    mod insert_account_id_tests {
        use http::Extensions;

        use crate::{common::id::{uuid7::Uuid7, AccountId}, middlewares::session::dsl::misc::insert_account_id};

        #[test]
        fn insert_one() {
            let mut extensions = Extensions::new();
            let account_id = AccountId::new(Uuid7::now());
            insert_account_id(&mut extensions, account_id.clone());
            assert_eq!(extensions.get::<AccountId>(), Some(&account_id));
        }
    }

    mod test_can_set_cookie_in_response_header_tests {
        use http::{header::SET_COOKIE, HeaderMap, HeaderValue};

        use crate::middlewares::session::dsl::misc::can_set_cookie_in_response_header;

        #[test]
        fn can_set() {
            let headers = HeaderMap::new();
            assert_eq!(can_set_cookie_in_response_header(&headers), true);
        }

        #[test]
        fn cannot_set() {
            let mut headers = HeaderMap::new();
            headers.insert(SET_COOKIE, HeaderValue::from_static("dummy"));
            assert_eq!(can_set_cookie_in_response_header(&headers), false);
        }
    }

    mod is_same_token_tests {
        use crate::{common::session::value::LoginToken, middlewares::session::dsl::misc::is_same_token};

        #[test]
        fn same() {
            let token = LoginToken::gen();
            assert_eq!(is_same_token(&token, &token), true);
        }

        #[test]
        fn different() {
            let token = LoginToken::gen();
            let another_token = LoginToken::gen();
            assert_eq!(is_same_token(&token, &another_token), false);
        }
    }

    mod should_extend_series_id_expiration_tests {
        use crate::middlewares::session::dsl::misc::{should_extend_series_id_expiration, SeriesIdRefreshTimestamp, UnixtimeMillis, SESSION_EXTENSION_THRESHOLD};

        #[test]
        fn within_threshold() {
            let current_unixtime = UnixtimeMillis::now();
            let within_threshold_unixtime = UnixtimeMillis::from(current_unixtime.value() - SESSION_EXTENSION_THRESHOLD + 86400);
            assert_eq!(should_extend_series_id_expiration(&SeriesIdRefreshTimestamp::new(within_threshold_unixtime)).unwrap(), false);
        }

        #[test]
        fn over_threshold() {
            let current_unixtime = UnixtimeMillis::now();
            let over_threshold_unixtime = UnixtimeMillis::from(current_unixtime.value() - SESSION_EXTENSION_THRESHOLD - 1);
            assert_eq!(should_extend_series_id_expiration(&SeriesIdRefreshTimestamp::new(over_threshold_unixtime)).unwrap(), true);
        }
    }
}