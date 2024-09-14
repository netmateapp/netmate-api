use bb8_redis::redis::cmd;
use redis::ToRedisArgs;

use crate::{common::{fallible::Fallible, id::AccountId, session::value::SessionId}, helper::redis::{Connection, TypedCommand, NAMESPACE_SEPARATOR}, middlewares::manage_session::{dsl::authenticate::{AuthenticateSession, AuthenticateSessionError}, interpreter::SESSION_ID_NAMESPACE}};

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

fn format_key(session_id: &SessionId) -> String {
    format!("{}{}{}", SESSION_ID_NAMESPACE, NAMESPACE_SEPARATOR, session_id.to_string())
}

impl<'a> ToRedisArgs for Key<'a> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        format_key(self.0).write_redis_args(out);
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

#[cfg(test)]
mod tests {
    use crate::{common::session::value::SessionId, helper::redis::NAMESPACE_SEPARATOR, middlewares::manage_session::interpreter::SESSION_ID_NAMESPACE};

    use super::format_key;

    #[test]
    fn test_format_key() {
        let session_id = SessionId::gen();
        let key = format_key(&session_id);
        let expected = format!("{}{}{}", SESSION_ID_NAMESPACE, NAMESPACE_SEPARATOR, session_id.to_string());
        assert_eq!(key, expected);
    }
}

