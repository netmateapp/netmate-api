use http::Response;
use thiserror::Error;

use crate::common::{fallible::Fallible, id::AccountId, session::{cookie::{set_refresh_pair_cookie_with_expiration, set_session_cookie_with_expiration}, refresh_pair_expiration::RefreshPairExpirationSeconds, session_expiration::SessionExpirationSeconds}};

use super::{assign_refresh_pair::AssignRefreshPair, assign_session_id::AssignSessionId};

pub(crate) trait StartSession {
    async fn start_session<B>(&self, session_account_id: &AccountId, mut response: Response<B>) -> Fallible<(), StartSessionError>
    where
        Self: AssignSessionId + AssignRefreshPair,
    {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> StartSessionError {
            StartSessionError::StartSessionFailed(e.into())
        }

        let session_id = self.assign_session_id(session_account_id, Self::session_expiration())
            .await
            .map_err(handle_error)?;
        set_session_cookie_with_expiration(&mut response, &session_id);

        let (session_series, refresh_token) = self.assign_refresh_pair(session_account_id, Self::refresh_pair_expiration())
            .await
            .map_err(handle_error)?;
        set_refresh_pair_cookie_with_expiration(&mut response, &session_series, &refresh_token);

        Ok(())
    }

    fn session_expiration() -> &'static SessionExpirationSeconds;

    fn refresh_pair_expiration() -> &'static RefreshPairExpirationSeconds;
}

#[derive(Debug, Error)]
pub enum StartSessionError {
    #[error("セッションの開始に失敗しました")]
    StartSessionFailed(#[source] anyhow::Error),
}