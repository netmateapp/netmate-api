use crate::{common::{fallible::Fallible, id::account_id::AccountId, session::{session_expiration::SessionExpirationSeconds, session_id::SessionId}}, helper::redis::TypedCommand, middlewares::{manage_session::dsl::update_session::{UpdateSession, UpdateSessionError}, session::{SessionIdKey, SetSessionIdCommand}}};

use super::ManageSessionImpl;

impl UpdateSession for ManageSessionImpl {
    async fn try_assign_new_session_id_with_expiration_if_unused(&self, new_session_id: &SessionId, session_account_id: AccountId, new_expiration: SessionExpirationSeconds) -> Fallible<(), UpdateSessionError> {
        let key = SessionIdKey(new_session_id);

        SetSessionIdCommand.run(&self.cache, (key, session_account_id, new_expiration))
            .await
            .map_err(UpdateSessionError::AssignNewSessionIdFailed)?
            .map_or_else(|| Err(UpdateSessionError::SessionIdAlreadyUsed), |_| Ok(()))
    }
}