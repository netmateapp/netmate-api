use redis::cmd;

use crate::{common::{fallible::Fallible, profile::account_id::AccountId, session::{session_expiration::SessionExpirationSeconds, session_id::SessionId}}, helper::redis::conn, middlewares::{session::SessionIdKey, start_session::dsl::assign_session_id::{AssignSessionId, AssignSessionIdError}}};

use super::StartSessionImpl;

impl AssignSessionId for StartSessionImpl {
    async fn try_assign_new_session_id_with_expiration_if_unused(&self, session_id: &SessionId, session_account_id: AccountId, expiration: SessionExpirationSeconds) -> Fallible<(), AssignSessionIdError> {
        let mut conn = conn(&self.cache, |e| AssignSessionIdError::AssignNewSessionIdFailed(e.into())).await?;

        cmd("SET")
            .arg(SessionIdKey::new(session_id))
            .arg(session_account_id)
            .arg("EX")
            .arg(expiration)
            .arg("NX")
            .query_async::<Option<()>>(&mut *conn) // 重複時はNone
            .await
            .map_err(|e| AssignSessionIdError::AssignNewSessionIdFailed(e.into()))?
            .map_or_else(|| Err(AssignSessionIdError::SessionIdAlreadyUsed), |_| Ok(()))
    }
}