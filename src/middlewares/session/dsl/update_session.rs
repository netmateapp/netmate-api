use thiserror::Error;

use crate::common::{fallible::Fallible, id::AccountId, session::value::SessionId};

pub(crate) trait UpdateSession {
    async fn update_session(&self, session_account_id: &AccountId, new_expiration: &SessionExpirationTime) -> Fallible<SessionId, UpdateSessionError> {
        let mut new_session_id = SessionId::gen();
        
        // このループは奇跡が起きない限りO(1)となる
        loop {
            match self.try_assign_new_session_id_with_expiration_to_account(&new_session_id, session_account_id, new_expiration).await {
                Ok(()) => return Ok(new_session_id),
                Err(UpdateSessionError::SessionIdAlreadyUsed) => new_session_id = SessionId::gen(),
                _ => return Err(UpdateSessionError::AssignNewSessionIdFailed)
            }
        }
    }

    async fn try_assign_new_session_id_with_expiration_to_account(&self, new_session_id: &SessionId, session_account_id: &AccountId, new_expiration: &SessionExpirationTime) -> Fallible<(), UpdateSessionError>;
}

pub struct SessionExpirationTime(u32);

impl SessionExpirationTime {
    pub const fn new(seconds: u32) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Error)]
pub enum UpdateSessionError {
    #[error("セッションIDが既に使用されています")]
    SessionIdAlreadyUsed,
    #[error("新規セッションIDの割り当てに失敗しました")]
    AssignNewSessionIdFailed,
}
