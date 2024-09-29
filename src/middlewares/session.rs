use redis::{RedisWrite, ToRedisArgs};

use crate::{common::{profile::account_id::AccountId, session::{refresh_token::RefreshToken, session_id::SessionId, session_series::SessionSeries}}, helper::redis::namespace::{Namespace, NAMESPACE_SEPARATOR}};

pub const SESSION_ID_NAMESPACE: Namespace = Namespace::of("sid");

pub const REFRESH_PAIR_NAMESPACE: Namespace = Namespace::of("rfp");
pub const REFRESH_PAIR_VALUE_SEPARATOR: char = '$';

pub struct SessionIdKey(String);

impl SessionIdKey {
    pub fn new(session_id: &SessionId) -> Self {
        Self(format!("{}{}{}", SESSION_ID_NAMESPACE, NAMESPACE_SEPARATOR, session_id))
    }
}

impl ToRedisArgs for SessionIdKey {
    fn write_redis_args<W: ?Sized + RedisWrite>(&self, out: &mut W) {
        self.0.write_redis_args(out);
    }
}

pub struct RefreshPairKey(String);

impl RefreshPairKey {
    pub fn new(session_series: &SessionSeries) -> Self {
        Self(format!("{}{}{}", REFRESH_PAIR_NAMESPACE, NAMESPACE_SEPARATOR, session_series))
    }
}


impl ToRedisArgs for RefreshPairKey {
    fn write_redis_args<W: ?Sized + RedisWrite>(&self, out: &mut W) {
        self.0.write_redis_args(out);
    }
}

pub struct RefreshPairValue(String);

impl RefreshPairValue {
    pub fn new(new_refresh_token: &RefreshToken, session_account_id: AccountId) -> Self {
        Self(format!("{}{}{}", new_refresh_token, REFRESH_PAIR_VALUE_SEPARATOR, session_account_id))
    }
}

impl ToRedisArgs for RefreshPairValue {
    fn write_redis_args<W: ?Sized + RedisWrite>(&self, out: &mut W) {
        self.0.write_redis_args(out);
    }
}

#[cfg(test)]
mod tests {
    use crate::{common::{profile::account_id::AccountId, session::{refresh_token::RefreshToken, session_id::SessionId, session_series::SessionSeries}}, helper::redis::namespace::NAMESPACE_SEPARATOR, middlewares::session::{RefreshPairKey, RefreshPairValue, SessionIdKey, REFRESH_PAIR_NAMESPACE, REFRESH_PAIR_VALUE_SEPARATOR, SESSION_ID_NAMESPACE}};

    #[test]
    fn test_format_session_id_key() {
        let session_id = SessionId::gen();
        let key = SessionIdKey::new(&session_id);
        let expected = format!("{}{}{}", SESSION_ID_NAMESPACE, NAMESPACE_SEPARATOR, session_id);
        assert_eq!(key.0, expected);
    }

    #[test]
    fn test_format_refresh_pair_key() {
        let session_series = SessionSeries::gen();
        let key = RefreshPairKey::new(&session_series);
        let expected = format!("{}{}{}", REFRESH_PAIR_NAMESPACE, NAMESPACE_SEPARATOR, session_series);
        assert_eq!(key.0, expected);
    }

    #[test]
    fn test_format_refresh_pair_value() {
        let new_refresh_token = RefreshToken::gen();
        let session_account_id = AccountId::gen();
        let value = RefreshPairValue::new(&new_refresh_token, session_account_id);
        let expected = format!("{}{}{}", new_refresh_token, REFRESH_PAIR_VALUE_SEPARATOR, session_account_id);
        assert_eq!(value.0, expected);
    }
}