use thiserror::Error;

use crate::common::{fallible::Fallible, profile::account_id::AccountId, session::session_id::SessionId};

pub(crate) trait AuthenticateSession {
    async fn authenticate_session(&self, session_id: &SessionId) -> Fallible<AccountId, AuthenticateSessionError> {
        self.resolve_session_id_to_account_id(session_id)
            .await?
            .ok_or_else(|| AuthenticateSessionError::InvalidSessionId)
    }

    async fn resolve_session_id_to_account_id(&self, session_id: &SessionId) -> Fallible<Option<AccountId>, AuthenticateSessionError>;
}

#[derive(Debug, Error)]
pub enum AuthenticateSessionError {
    #[error("セッションIDの解決に失敗しました")]
    ResolveSessionIdFailed(#[source] anyhow::Error),
    #[error("無効なセッションIDです")]
    InvalidSessionId,
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use crate::common::{fallible::Fallible, profile::account_id::AccountId, session::session_id::SessionId, uuid::uuid7::Uuid7};

    use super::{AuthenticateSession, AuthenticateSessionError};

    static VALID_SESSION_ID: LazyLock<SessionId> = LazyLock::new(SessionId::gen);

    struct MockAuthenticateUser;

    impl AuthenticateSession for MockAuthenticateUser {
        async fn resolve_session_id_to_account_id(&self, session_id: &SessionId) -> Fallible<Option<AccountId>, AuthenticateSessionError> {
            if session_id == &*VALID_SESSION_ID {
                Ok(Some(AccountId::of(Uuid7::now())))
            } else {
                Ok(None)
            }
        }
    }

    #[tokio::test]
    async fn valid_session_id() {
        let result = MockAuthenticateUser.authenticate_session(&VALID_SESSION_ID).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn invalid_session_id() {
        let result = MockAuthenticateUser.authenticate_session(&SessionId::gen()).await;
        assert!(result.is_err());
    }
}