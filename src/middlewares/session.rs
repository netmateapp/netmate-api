use redis::{cmd, RedisWrite, ToRedisArgs};

use crate::{common::{id::account_id::AccountId, session::{refresh_token::RefreshToken, session_expiration::SessionExpirationSeconds, session_id::SessionId, session_series::SessionSeries}}, helper::redis::{Connection, Namespace, TypedCommand, EX_OPTION, NAMESPACE_SEPARATOR, NX_OPTION, SET_COMMAND}};

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

impl<'a> TypedCommand<(SessionIdOld<'a>, AccountId, SessionExpirationSeconds), Option<()>> for SetSessionIdCommand {
    async fn execute(&self, mut conn: Connection<'_>, (key, session_account_id, new_expiration): (SessionIdOld<'a>, AccountId, SessionExpirationSeconds)) -> anyhow::Result<Option<()>> {
        cmd(SET_COMMAND)
            .arg(key)
            .arg(session_account_id.to_string())
            .arg(EX_OPTION)
            .arg(new_expiration.as_secs())
            .arg(NX_OPTION)
            .query_async::<Option<()>>(&mut *conn) // 重複が無ければSome(())、あればNone
            .await
            .map_err(anyhow::Error::from)
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
    use crate::{common::{id::account_id::AccountId, session::{refresh_token::RefreshToken, session_id::SessionId, session_series::SessionSeries}}, helper::redis::NAMESPACE_SEPARATOR, middlewares::session::{format_refresh_pair_key, format_refresh_pair_value, format_session_id_key, REFRESH_PAIR_NAMESPACE, REFRESH_PAIR_VALUE_SEPARATOR, SESSION_ID_NAMESPACE}};

    #[test]
    fn test_format_session_id_key() {
        let session_id = SessionId::gen();
        let key = format_session_id_key(&session_id);
        let expected = format!("{}{}{}", SESSION_ID_NAMESPACE, NAMESPACE_SEPARATOR, session_id);
        assert_eq!(key, expected);
    }

    #[test]
    fn test_format_refresh_pair_key() {
        let session_series = SessionSeries::gen();
        let key = format_refresh_pair_key(&session_series);
        let expected = format!("{}{}{}", REFRESH_PAIR_NAMESPACE, NAMESPACE_SEPARATOR, session_series);
        assert_eq!(key, expected);
    }

    #[test]
    fn test_format_refresh_pair_value() {
        let new_refresh_token = RefreshToken::gen();
        let session_account_id = AccountId::gen();
        let value = format_refresh_pair_value(&new_refresh_token, session_account_id);
        let expected = format!("{}{}{}", new_refresh_token, REFRESH_PAIR_VALUE_SEPARATOR, session_account_id);
        assert_eq!(value, expected);
    }
}