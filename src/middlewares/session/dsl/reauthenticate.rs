use crate::common::{fallible::Fallible, id::AccountId, session::value::{RefreshToken, SessionSeries}};

pub(crate) trait ReAuthenticateSession {
    async fn reauthenticate_session(&self, session_series: &SessionSeries, refresh_token: &RefreshToken) -> Fallible<AccountId, ReAuthenticateUserError> {
        match self.fetch_refresh_token_and_account_id(&session_series).await? {
            Some((stored_refresh_token, account_id)) => {
                if refresh_token == &stored_refresh_token {
                    Ok(account_id)
                } else {
                    Err(ReAuthenticateUserError::PotentialSessionTheft)
                }
            },
            None => Err(ReAuthenticateUserError::InvalidRefreshToken)
        }
    }

    async fn fetch_refresh_token_and_account_id(&self, session_series: &SessionSeries) -> Fallible<Option<(RefreshToken, AccountId)>, ReAuthenticateUserError>;
}

pub enum ReAuthenticateUserError {
    UpdateSessionFailed,
    InvalidRefreshToken,
    PotentialSessionTheft,
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use crate::common::{fallible::Fallible, id::{uuid7::Uuid7, AccountId}, session::value::{RefreshToken, SessionSeries}};

    use super::{ReAuthenticateSession, ReAuthenticateUserError};

    static REAUTHENTICATED: LazyLock<SessionSeries> = LazyLock::new(|| SessionSeries::gen());
    static POTENTIAL_SESSION_THEFT: LazyLock<SessionSeries> = LazyLock::new(|| SessionSeries::gen());

    struct MockReauthenticateSession;

    impl ReAuthenticateSession for MockReauthenticateSession {
        async fn fetch_refresh_token_and_account_id(&self, session_series: &SessionSeries) -> Fallible<Option<(RefreshToken, AccountId)>, ReAuthenticateUserError> {
            if session_series == &*REAUTHENTICATED {
                Ok(Some((RefreshToken::gen(), AccountId::new(Uuid7::now()))))
            } else if session_series == &*POTENTIAL_SESSION_THEFT {
                Err(ReAuthenticateUserError::PotentialSessionTheft)
            } else {
                Err(ReAuthenticateUserError::InvalidRefreshToken)
            }
        }
    }

    #[tokio::test]
    async fn reauthenticated() {
        let result = MockReauthenticateSession.fetch_refresh_token_and_account_id(&*REAUTHENTICATED).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn potential_session_theft() {
        let result = MockReauthenticateSession.fetch_refresh_token_and_account_id(&*POTENTIAL_SESSION_THEFT).await;
        match result.err() {
            Some(ReAuthenticateUserError::PotentialSessionTheft) => (),
            _ => panic!()
        }
    }

    #[tokio::test]
    async fn invalid_refresh_token() {
        let result = MockReauthenticateSession.fetch_refresh_token_and_account_id(&SessionSeries::gen()).await;
        match result.err() {
            Some(ReAuthenticateUserError::InvalidRefreshToken) => (),
            _ => panic!()
        }
    }
}