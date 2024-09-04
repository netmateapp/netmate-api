use std::time::{SystemTime, UNIX_EPOCH};

use http::{header::SET_COOKIE, Extensions, HeaderMap};

use crate::{common::{fallible::Fallible, id::AccountId, session::value::LoginToken}, middlewares::session::dsl::ManageSessionError};

pub fn insert_account_id(extensions: &mut Extensions, account_id: AccountId) {
    extensions.insert(account_id);
}

pub fn can_set_cookie_in_response_header(headers: &HeaderMap) -> bool {
    !headers.contains_key(SET_COOKIE)
}

pub fn is_same_token(request_token: &LoginToken, registered_token: &LoginToken) -> bool {
    request_token.value().value() == registered_token.value().value()
}

const SESSION_EXTENSION_THRESHOLD: u64 = 30 * 24 * 60 * 60;

pub fn should_extend_series_id_expiration(last_series_id_expiration_update_time: &UnixtimeSeconds) -> Fallible<bool, ManageSessionError> {
    let current_unixtime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| ManageSessionError::CheckSeriesIdExpirationExtendabilityFailed(e.into()))?;

    Ok(current_unixtime - last_series_id_expiration_update_time.value() > SESSION_EXTENSION_THRESHOLD)
}

pub struct UnixtimeSeconds(u64);

impl UnixtimeSeconds {
    pub fn new(unixtime_seconds: u64) -> Self {
        Self(unixtime_seconds)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}


#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use http::Extensions;

    use crate::{common::{id::AccountId, session::value::LoginToken}, middlewares::session::dsl::misc::{can_set_cookie_in_response_header, is_same_token, should_extend_series_id_expiration, UnixtimeSeconds, SESSION_EXTENSION_THRESHOLD}};

    #[test]
    fn test_insert_account_id() {
        let mut extensions = Extensions::new();
        let account_id = AccountId::gen();
        super::insert_account_id(&mut extensions, account_id.clone());
        assert_eq!(extensions.get::<AccountId>(), Some(&account_id));
    }

    #[test]
    fn test_can_set_cookie_in_response_header() {
        let mut headers = http::HeaderMap::new();
        assert_eq!(can_set_cookie_in_response_header(&headers), true);

        headers.insert(http::header::SET_COOKIE, http::HeaderValue::from_static("dummy"));
        assert_eq!(can_set_cookie_in_response_header(&headers), false);
    }

    #[test]
    fn test_is_same_token() {
        let token = LoginToken::gen();
        assert_eq!(is_same_token(&token, &token), true);
    }

    #[test]
    fn test_should_extend_series_id_expiration() {
        let current_unixtime = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let within_threshold_unixtime = UnixtimeSeconds::new(current_unixtime - SESSION_EXTENSION_THRESHOLD + 10);
        let over_threshold_unixtime = UnixtimeSeconds::new(current_unixtime - SESSION_EXTENSION_THRESHOLD - 1);

        assert_eq!(should_extend_series_id_expiration(&within_threshold_unixtime).unwrap(), false);
        assert_eq!(should_extend_series_id_expiration(&over_threshold_unixtime).unwrap(), true);
    }
}