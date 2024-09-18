use redis::cmd;

use crate::{common::{fallible::Fallible, id::account_id::AccountId, session::{session_expiration::SessionExpirationSeconds, session_id::SessionId}}, helper::redis::conn, middlewares::{manage_session::dsl::update_session::{UpdateSession, UpdateSessionError}, session::SessionIdKey}};

use super::ManageSessionImpl;

impl UpdateSession for ManageSessionImpl {
    async fn try_assign_new_session_id_with_expiration_if_unused(&self, new_session_id: &SessionId, session_account_id: AccountId, new_expiration: SessionExpirationSeconds) -> Fallible<(), UpdateSessionError> {
        let mut conn = conn(&self.cache, |e| UpdateSessionError::AssignNewSessionIdFailed(e.into())).await?;
        
        cmd("SET")
            .arg(SessionIdKey::new(new_session_id))
            .arg(session_account_id)
            .arg("EX")
            .arg(new_expiration)
            .arg("NX")
            .query_async::<Option<()>>(&mut *conn) // 重複時はNoneを返す
            .await
            .map_err(|e| UpdateSessionError::AssignNewSessionIdFailed(e.into()))?
            .map_or_else(|| Err(UpdateSessionError::SessionIdAlreadyUsed), |_| Ok(()))
    }
}