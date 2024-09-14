use bb8_redis::redis::cmd;
use redis::ToRedisArgs;

use crate::{common::{fallible::Fallible, id::AccountId, session::value::SessionId}, helper::redis::{Connection, TypedCommand, EX_OPTION, NAMESPACE_SEPARATOR, NX_OPTION, SET_COMMAND}, middlewares::manage_session::{dsl::{manage_session::SessionExpirationSeconds, update_session::{UpdateSession, UpdateSessionError}}, interpreter::SESSION_ID_NAMESPACE}};

use super::ManageSessionImpl;

impl UpdateSession for ManageSessionImpl {
    async fn try_assign_new_session_id_with_expiration_if_unused(&self, new_session_id: &SessionId, session_account_id: &AccountId, new_expiration: &SessionExpirationSeconds) -> Fallible<(), UpdateSessionError> {
        let key = Key(new_session_id);

        SetNewSessionIdCommand.run(&self.cache, (key, session_account_id, new_expiration))
            .await
            .map_err(|e| UpdateSessionError::AssignNewSessionIdFailed(e.into()))?
            .map_or_else(|| Err(UpdateSessionError::SessionIdAlreadyUsed), |_| Ok(()))
    }
}

struct SetNewSessionIdCommand;

struct Key<'a>(&'a SessionId);

fn format_key(session_id: &SessionId) -> String {
    format!("{}{}{}", SESSION_ID_NAMESPACE, NAMESPACE_SEPARATOR, session_id)
}

impl<'a> ToRedisArgs for Key<'a> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        format_key(self.0).write_redis_args(out);
    }
}

impl<'a, 'b, 'c> TypedCommand<(Key<'a>, &'b AccountId, &'c SessionExpirationSeconds), Option<()>> for SetNewSessionIdCommand {
    async fn execute(&self, mut conn: Connection<'_>, (key, session_account_id, new_expiration): (Key<'a>, &'b AccountId, &'c SessionExpirationSeconds)) -> anyhow::Result<Option<()>> {
        cmd(SET_COMMAND)
            .arg(key)
            .arg(session_account_id.to_string())
            .arg(EX_OPTION)
            .arg(new_expiration.as_secs())
            .arg(NX_OPTION)
            .query_async::<Option<()>>(&mut *conn)
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
        let expected = format!("{}{}{}", SESSION_ID_NAMESPACE, NAMESPACE_SEPARATOR, session_id);
        assert_eq!(key, expected);
    }
}