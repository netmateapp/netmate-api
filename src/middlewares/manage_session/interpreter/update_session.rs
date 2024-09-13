use bb8_redis::redis::cmd;

use crate::{common::{fallible::Fallible, id::AccountId, session::value::SessionId}, helper::valkey::conn, middlewares::manage_session::{dsl::{manage_session::SessionExpirationSeconds, update_session::{UpdateSession, UpdateSessionError}}, interpreter::SESSION_ID_NAMESPACE}};

use super::ManageSessionImpl;

impl UpdateSession for ManageSessionImpl {
    async fn try_assign_new_session_id_with_expiration_if_unused(&self, new_session_id: &SessionId, session_account_id: &AccountId, new_expiration: &SessionExpirationSeconds) -> Fallible<(), UpdateSessionError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> UpdateSessionError {
            UpdateSessionError::AssignNewSessionIdFailed(e.into())
        }

        let mut conn = conn(&self.cache, handle_error)
            .await?;

        let key = format!("{}:{}", SESSION_ID_NAMESPACE, new_session_id.to_string());

        cmd("SET")
            .arg(key)
            .arg(session_account_id.to_string())
            .arg("EX")
            .arg(new_expiration.as_secs())
            .arg("NX")
            .query_async::<Option<()>>(&mut *conn)
            .await
            .map_err(handle_error)?
            .map_or_else(|| Err(UpdateSessionError::SessionIdAlreadyUsed), |_| Ok(()))
    }
}