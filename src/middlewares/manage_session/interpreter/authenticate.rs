use bb8_redis::redis::cmd;
use redis::ToRedisArgs;

use crate::{common::{fallible::Fallible, id::account_id::AccountId, session::session_id::SessionId}, helper::redis::{Connection, TypedCommand}, middlewares::{manage_session::dsl::authenticate::{AuthenticateSession, AuthenticateSessionError}, value::format_session_id_key}};

use super::ManageSessionImpl;

impl AuthenticateSession for ManageSessionImpl {
    async fn resolve_session_id_to_account_id(&self, session_id: &SessionId) -> Fallible<Option<AccountId>, AuthenticateSessionError> {
        GetAccountIdCommand.run(&self.cache, Key(session_id))
            .await
            .map_err(|e| AuthenticateSessionError::ResolveSessionIdFailed(e.into()))
    }
}

struct GetAccountIdCommand;

struct Key<'a>(&'a SessionId);

impl<'a> ToRedisArgs for Key<'a> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        format_session_id_key(self.0).write_redis_args(out);
    }
}

impl<'a> TypedCommand<Key<'a>, Option<AccountId>> for GetAccountIdCommand {
    async fn execute(&self, mut conn: Connection<'_>, key: Key<'a>) -> anyhow::Result<Option<AccountId>> {
        cmd("GET")
            .arg(key)
            .query_async::<Option<AccountId>>(&mut *conn)
            .await
            .map_err(Into::into)
    }
}