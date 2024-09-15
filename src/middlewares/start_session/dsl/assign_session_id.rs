use thiserror::Error;

use crate::common::{fallible::Fallible, id::AccountId, session::{session_expiration::SessionExpirationSeconds, session_id::SessionId}};

pub(crate)  trait AssignSessionId {
    async fn assign_session_id(&self, session_account_id: &AccountId, expiration: &SessionExpirationSeconds) -> Fallible<SessionId, AssignSessionIdError> {
        let mut session_id = SessionId::gen();
        
        // このループは奇跡が起きない限りO(1)となる
        loop {
            match self.try_assign_new_session_id_with_expiration_if_unused(&session_id, session_account_id, expiration).await {
                Ok(()) => return Ok(session_id),
                Err(AssignSessionIdError::SessionIdAlreadyUsed) => session_id = SessionId::gen(),
                Err(e) => return Err(e),
            }
        }
    }

    async fn try_assign_new_session_id_with_expiration_if_unused(&self, session_id: &SessionId, session_account_id: &AccountId, expiration: &SessionExpirationSeconds) -> Fallible<(), AssignSessionIdError>;
}

#[derive(Debug, Error)]
pub enum AssignSessionIdError {
    #[error("セッションIDが既に使用されています")]
    SessionIdAlreadyUsed,
    #[error("新規セッションIDの割り当てに失敗しました")]
    AssignNewSessionIdFailed(#[source] anyhow::Error),
}
