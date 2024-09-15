use crate::{common::{fallible::Fallible, id::account_id::AccountId, session::{session_expiration::SessionExpirationSeconds, session_id::SessionId}}, helper::redis::TypedCommand, middlewares::{session::{SessionIdKey, SetSessionIdCommand}, start_session::dsl::assign_session_id::{AssignSessionId, AssignSessionIdError}}};

use super::StartSessionImpl;

impl AssignSessionId for StartSessionImpl {
    async fn try_assign_new_session_id_with_expiration_if_unused(&self, session_id: &SessionId, session_account_id: AccountId, expiration: SessionExpirationSeconds) -> Fallible<(), AssignSessionIdError> {
        let key = SessionIdKey(session_id);

        SetSessionIdCommand.run(&self.cache, (key, session_account_id, expiration))
            .await
            .map_err(|e| AssignSessionIdError::AssignNewSessionIdFailed(e.into()))?
            .map_or_else(|| Err(AssignSessionIdError::SessionIdAlreadyUsed), |_| Ok(()))
    }
}