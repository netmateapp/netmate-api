use std::str::FromStr;

use bb8_redis::redis::cmd;
use redis::{FromRedisValue, RedisError, RedisResult, ToRedisArgs};
use uuid::Uuid;

use crate::{common::{fallible::Fallible, id::account_id::AccountId, session::{refresh_token::RefreshToken, session_series::SessionSeries}, uuid::uuid7::Uuid7}, helper::redis::{Connection, TypedCommand, GET_COMMAND}, middlewares::{manage_session::dsl::reauthenticate::{ReAuthenticateSession, ReAuthenticateSessionError}, value::{format_refresh_pair_key, REFRESH_PAIR_VALUE_SEPARATOR}}};

use super::ManageSessionImpl;

impl ReAuthenticateSession for ManageSessionImpl {
    async fn fetch_refresh_token_and_account_id(&self, session_series: &SessionSeries) -> Fallible<Option<(RefreshToken, AccountId)>, ReAuthenticateSessionError> {
        let key = Key(session_series);

        GetRefreshPairCommand.run(&self.cache, key)
            .await
            .map(|o| o.map(|p| (p.0, p.1)))
            .map_err(|e| ReAuthenticateSessionError::FetchRefreshTokenAndAccountIdFailed(e.into()))
    }
}

struct GetRefreshPairCommand;

struct Key<'a>(&'a SessionSeries);

impl<'a> ToRedisArgs for Key<'a> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        format_refresh_pair_key(self.0).write_redis_args(out);
    }
}

struct RefreshPair(RefreshToken, AccountId);

impl FromRedisValue for RefreshPair {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        fn handle_error() -> RedisError {
            RedisError::from((redis::ErrorKind::TypeError, "リフレッシュペアの形式を満たしていません"))
        }

        let value = String::from_redis_value(v)?;
        let mut parts = value.splitn(2, REFRESH_PAIR_VALUE_SEPARATOR);

        let token = parts.next()
            .ok_or_else(handle_error)
            .map(|s| RefreshToken::from_str(s))?
            .map_err(|_| handle_error())?;

        let account_id = parts.next()
            .ok_or_else(handle_error)
            .map(|s| Uuid::from_str(s))?
            .map_err(|_| handle_error())
            .map(|u| Uuid7::try_from(u))?
            .map_err(|_| handle_error())
            .map(|u| AccountId::of(u))?;

        Ok(RefreshPair(token, account_id))
    }
}

impl<'a> TypedCommand<Key<'a>, Option<RefreshPair>> for GetRefreshPairCommand {
    async fn execute(&self, mut conn: Connection<'_>, key: Key<'a>) -> anyhow::Result<Option<RefreshPair>> {
        cmd(GET_COMMAND).arg(key)
            .query_async::<Option<RefreshPair>>(&mut *conn)
            .await
            .map_err(Into::into)
    }
}
