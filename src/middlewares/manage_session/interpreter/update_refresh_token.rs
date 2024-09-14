use bb8_redis::redis::cmd;
use redis::ToRedisArgs;

use crate::{common::{fallible::Fallible, id::AccountId, session::value::{RefreshToken, SessionSeries}}, helper::redis::{Connection, TypedCommand, EX_OPTION, NAMESPACE_SEPARATOR, SET_COMMAND}, middlewares::manage_session::{dsl::{manage_session::RefreshPairExpirationSeconds, update_refresh_token::{UpdateRefreshToken, UpdateRefreshTokenError}}, interpreter::{REFRESH_PAIR_NAMESPACE, REFRESH_PAIR_VALUE_SEPARATOR}}};

use super::ManageSessionImpl;

impl UpdateRefreshToken for ManageSessionImpl {
    async fn assign_new_refresh_token_with_expiration(&self, new_refresh_token: &RefreshToken, session_series: &SessionSeries, session_account_id: &AccountId, expiration: &RefreshPairExpirationSeconds) -> Fallible<(), UpdateRefreshTokenError> {
        let key = Key(session_series);
        let value = Value(new_refresh_token, session_account_id);

        SetNewRefreshToken::run(&self.cache, (key, value, expiration))
            .await
            .map_err(|e| UpdateRefreshTokenError::AssignNewRefreshTokenFailed(e.into()))
    }
}

struct SetNewRefreshToken;

struct Key<'a>(&'a SessionSeries);

impl<'a> ToRedisArgs for Key<'a> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        let key = format!("{}{}{}", REFRESH_PAIR_NAMESPACE, NAMESPACE_SEPARATOR, self.0);
        key.write_redis_args(out);
    }
}

struct Value<'a, 'b>(&'a RefreshToken, &'b AccountId);

impl<'a, 'b> ToRedisArgs for Value<'a, 'b> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        let key = format!("{}{}{}", self.0, REFRESH_PAIR_VALUE_SEPARATOR, self.1);
        key.write_redis_args(out);
    }
}

impl<'a, 'b, 'c, 'd> TypedCommand<(Key<'a>, Value<'b, 'c>, &'d RefreshPairExpirationSeconds), ()> for SetNewRefreshToken {
    async fn execute(mut conn: Connection<'_>, args: (Key<'a>, Value<'b, 'c>, &'d RefreshPairExpirationSeconds)) -> anyhow::Result<()> {
        cmd(SET_COMMAND)
            .arg(args.0)
            .arg(args.1)
            .arg(EX_OPTION)
            .arg(args.2)
            .query_async(&mut *conn)
            .await
            .map_err(Into::into)
    }
}