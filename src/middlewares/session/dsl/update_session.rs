use crate::common::{fallible::Fallible, id::AccountId, session::value::SessionId};

pub struct SessionExpirationTime(u32);

impl SessionExpirationTime {
    pub const fn new(seconds: u32) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

pub(crate) trait UpdateSession {
    async fn update_session(&self, session_account_id: &AccountId, new_expiration: &SessionExpirationTime) -> Fallible<SessionId, UpdateSessionError> {
        let mut new_session_id = SessionId::gen();
        
        // このループは奇跡が起きない限りO(1)となる
        loop {
            match self.try_activate_new_session_id_with_expiration(&new_session_id, session_account_id, new_expiration).await {
                Ok(()) => return Ok(new_session_id),
                Err(UpdateSessionError::SessionIdAlreadyUsed) => new_session_id = SessionId::gen(),
                _ => return Err(UpdateSessionError::IssueNewSessionIdFailed)
            }
        }
    }

    async fn try_activate_new_session_id_with_expiration(&self, new_session_id: &SessionId, session_account_id: &AccountId, new_expiration: &SessionExpirationTime) -> Fallible<(), UpdateSessionError>;
}

pub enum UpdateSessionError {
    SessionIdAlreadyUsed,
    IssueNewSessionIdFailed,
}
