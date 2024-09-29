use thiserror::Error;

use crate::common::{fallible::Fallible, profile::account_id::AccountId, session::{refresh_token::RefreshToken, session_series::SessionSeries}};

pub(crate) trait ReAuthenticateSession {
    async fn reauthenticate_session(&self, session_series: &SessionSeries, refresh_token: RefreshToken) -> Fallible<AccountId, ReAuthenticateSessionError> {
        match self.fetch_refresh_token_and_account_id(session_series).await? {
            Some((stored_refresh_token, account_id)) => {
                if refresh_token == stored_refresh_token {
                    Ok(account_id)
                } else {
                    Err(ReAuthenticateSessionError::PotentialSessionTheft(account_id))
                }
            },
            None => Err(ReAuthenticateSessionError::InvalidRefreshToken)
        }
    }

    async fn fetch_refresh_token_and_account_id(&self, session_series: &SessionSeries) -> Fallible<Option<(RefreshToken, AccountId)>, ReAuthenticateSessionError>;
}

#[derive(Debug, Error)]
pub enum ReAuthenticateSessionError {
    #[error("リフレッシュトークンとアカウントIDの取得に失敗しました")]
    FetchRefreshTokenAndAccountIdFailed(#[source] anyhow::Error),
    #[error("無効なリフレッシュトークンです")]
    InvalidRefreshToken,
    #[error("セッションの盗用の可能性があります")]
    PotentialSessionTheft(AccountId),
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use crate::common::{fallible::Fallible, profile::account_id::AccountId, session::{refresh_token::RefreshToken, session_series::SessionSeries}};

    use super::{ReAuthenticateSession, ReAuthenticateSessionError};

    static REAUTHENTICATED: LazyLock<SessionSeries> = LazyLock::new(SessionSeries::gen);
    static POTENTIAL_SESSION_THEFT: LazyLock<SessionSeries> = LazyLock::new(SessionSeries::gen);

    struct MockReauthenticateSession;

    impl ReAuthenticateSession for MockReauthenticateSession {
        async fn fetch_refresh_token_and_account_id(&self, session_series: &SessionSeries) -> Fallible<Option<(RefreshToken, AccountId)>, ReAuthenticateSessionError> {
            if session_series == &*REAUTHENTICATED {
                Ok(Some((RefreshToken::gen(), AccountId::gen())))
            } else if session_series == &*POTENTIAL_SESSION_THEFT {
                Err(ReAuthenticateSessionError::PotentialSessionTheft(AccountId::gen()))
            } else {
                Err(ReAuthenticateSessionError::InvalidRefreshToken)
            }
        }
    }

    #[tokio::test]
    async fn reauthenticated() {
        let result = MockReauthenticateSession.fetch_refresh_token_and_account_id(&REAUTHENTICATED).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn potential_session_theft() {
        let result = MockReauthenticateSession.fetch_refresh_token_and_account_id(&POTENTIAL_SESSION_THEFT).await;
        match result.err() {
            Some(ReAuthenticateSessionError::PotentialSessionTheft(_)) => (),
            _ => panic!()
        }
    }

    #[tokio::test]
    async fn invalid_refresh_token() {
        let result = MockReauthenticateSession.fetch_refresh_token_and_account_id(&SessionSeries::gen()).await;
        match result.err() {
            Some(ReAuthenticateSessionError::InvalidRefreshToken) => (),
            _ => panic!()
        }
    }
}