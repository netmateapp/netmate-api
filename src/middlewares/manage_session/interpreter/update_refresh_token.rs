use bb8_redis::redis::cmd;
use redis::ToRedisArgs;

use crate::{common::{fallible::Fallible, id::account_id::AccountId, session::{refresh_pair_expiration::RefreshPairExpirationSeconds, refresh_token::RefreshToken, session_series::SessionSeries}}, helper::redis::{Connection, TypedCommand, EX_OPTION, NAMESPACE_SEPARATOR, SET_COMMAND}, middlewares::manage_session::{dsl::update_refresh_token::{UpdateRefreshToken, UpdateRefreshTokenError}, interpreter::{REFRESH_PAIR_NAMESPACE, REFRESH_PAIR_VALUE_SEPARATOR}}};

use super::ManageSessionImpl;

impl UpdateRefreshToken for ManageSessionImpl {
    async fn assign_new_refresh_token_with_expiration(&self, new_refresh_token: &RefreshToken, session_series: &SessionSeries, session_account_id: AccountId, expiration: RefreshPairExpirationSeconds) -> Fallible<(), UpdateRefreshTokenError> {
        let key = Key(session_series);
        let value = Value(new_refresh_token, session_account_id);

        SetNewRefreshTokenCommand.run(&self.cache, (key, value, expiration))
            .await
            .map_err(|e| UpdateRefreshTokenError::AssignNewRefreshTokenFailed(e.into()))
    }
}

struct SetNewRefreshTokenCommand;

struct Key<'a>(&'a SessionSeries);

fn format_key(session_series: &SessionSeries) -> String {
    format!("{}{}{}", REFRESH_PAIR_NAMESPACE, NAMESPACE_SEPARATOR, session_series)
}

impl<'a> ToRedisArgs for Key<'a> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        format_key(self.0).write_redis_args(out);
    }
}

struct Value<'a>(&'a RefreshToken, AccountId);

fn format_value(new_refresh_token: &RefreshToken, session_account_id: AccountId) -> String {
    format!("{}{}{}", new_refresh_token, REFRESH_PAIR_VALUE_SEPARATOR, session_account_id)
}

impl<'a> ToRedisArgs for Value<'a> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        format_value(self.0, self.1).write_redis_args(out);
    }
}

impl<'a, 'b> TypedCommand<(Key<'a>, Value<'b>, RefreshPairExpirationSeconds), ()> for SetNewRefreshTokenCommand {
    async fn execute(&self, mut conn: Connection<'_>, (key, value, expiration): (Key<'a>, Value<'b>, RefreshPairExpirationSeconds)) -> anyhow::Result<()> {
        cmd(SET_COMMAND)
            .arg(key)
            .arg(value)
            .arg(EX_OPTION)
            .arg(expiration)
            .query_async(&mut *conn)
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use crate::{common::{id::account_id::AccountId, session::{refresh_token::RefreshToken, session_series::SessionSeries}}, helper::redis::NAMESPACE_SEPARATOR, middlewares::manage_session::interpreter::{update_refresh_token::format_value, REFRESH_PAIR_NAMESPACE, REFRESH_PAIR_VALUE_SEPARATOR}};

    use super::format_key;

    #[test]
    fn test_format_key() {
        let session_series = SessionSeries::gen();
        let key = format_key(&session_series);
        assert_eq!(key, format!("{}{}{}", REFRESH_PAIR_NAMESPACE, NAMESPACE_SEPARATOR, session_series));
    }

    #[test]
    fn test_format_value() {
        let refresh_token = RefreshToken::gen();
        let account_id = AccountId::gen();
        let value = format_value(&refresh_token, account_id);
        assert_eq!(value, format!("{}{}{}", refresh_token, REFRESH_PAIR_VALUE_SEPARATOR, account_id));
    }
}