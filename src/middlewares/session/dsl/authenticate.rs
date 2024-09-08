use http::Request;
use thiserror::Error;

use crate::common::{fallible::Fallible, id::AccountId, session::value::SessionId};

pub(crate) trait AuthenticateUser {
    async fn authenticate_user<B>(&self, request: &mut Request<B>, session_id: &SessionId) -> Fallible<(), AuthenticateUserError> {
        self.resolve_session_id_to_account_id(&session_id)
            .await?
            .map(|account_id| request.extensions_mut().insert(account_id))
            .map(|_| ())
            .ok_or_else(|| AuthenticateUserError::InvalidSessionId)
    }

    async fn resolve_session_id_to_account_id(&self, session_id: &SessionId) -> Fallible<Option<AccountId>, AuthenticateUserError>;
}

#[derive(Debug, Error)]
pub enum AuthenticateUserError {
    #[error("セッションIDの解決に失敗しました")]
    ResolveSessionIdFailed,
    #[error("無効なセッションIDです")]
    InvalidSessionId,
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use http::Request;

    use crate::common::{fallible::Fallible, id::{uuid7::Uuid7, AccountId}, session::value::SessionId};

    use super::{AuthenticateUser, AuthenticateUserError};

    static VALID_SESSION_ID: LazyLock<SessionId> = LazyLock::new(|| SessionId::gen());

    struct MockAuthenticateUser;

    impl AuthenticateUser for MockAuthenticateUser {
        async fn resolve_session_id_to_account_id(&self, session_id: &SessionId) -> Fallible<Option<AccountId>, AuthenticateUserError> {
            if session_id == &*VALID_SESSION_ID {
                Ok(Some(AccountId::new(Uuid7::now())))
            } else {
                Err(AuthenticateUserError::ResolveSessionIdFailed)
            }
        }
    }

    #[tokio::test]
    async fn valid_session_id() {
        let mut request = Request::new(());
        let result = MockAuthenticateUser.authenticate_user(&mut request, &*VALID_SESSION_ID).await;
        let is_resolved = request.extensions().get::<AccountId>().is_some();
        assert!(result.is_ok() && is_resolved);
    }

    #[tokio::test]
    async fn invalid_session_id() {
        let mut request = Request::new(());
        let result = MockAuthenticateUser.authenticate_user(&mut request, &SessionId::gen()).await;
        let is_resolved = request.extensions().get::<AccountId>().is_some();
        assert!(result.is_err() && !is_resolved);
    }
}