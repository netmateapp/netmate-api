use std::convert::Infallible;

use http::{Request, Response};
use thiserror::Error;
use tower::Service;

use crate::common::{fallible::Fallible, id::account_id::AccountId, session::{cookie::{set_refresh_pair_cookie_with_expiration, set_session_cookie_with_expiration}, refresh_pair_expiration::REFRESH_PAIR_EXPIRATION, session_expiration::SESSION_EXPIRATION}};

use super::{assign_refresh_pair::AssignRefreshPair, assign_session_id::AssignSessionId};

pub(crate) trait StartSession {
    async fn start_session<S, B>(&self, inner: &mut S, request: Request<B>) -> Fallible<S::Response, StartSessionError>
    where
        Self: AssignSessionId + AssignRefreshPair,
        S: Service<Request<B>, Error = Infallible, Response = Response<B>>,
    {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> StartSessionError {
            StartSessionError::StartSessionFailed(e.into())
        }

        // `Infallible`であるため`unwrap`しても問題ない
        let mut response = inner.call(request)
            .await
            .unwrap();

        let session_account_id = response.extensions()
            .get::<AccountId>()
            .cloned();

        match session_account_id {
            Some(session_account_id) => {
                let session_id = self.assign_session_id(session_account_id, SESSION_EXPIRATION)
                    .await
                    .map_err(handle_error)?;
                set_session_cookie_with_expiration(&mut response, &session_id);
        
                let (session_series, refresh_token) = self.assign_refresh_pair(session_account_id, REFRESH_PAIR_EXPIRATION)
                    .await
                    .map_err(handle_error)?;
                set_refresh_pair_cookie_with_expiration(&mut response, &session_series, &refresh_token);
                
                Ok(response)
            },
            None => Ok(response)
        }
    }
}

#[derive(Debug, Error)]
pub enum StartSessionError {
    #[error("セッションの開始に失敗しました")]
    StartSessionFailed(#[source] anyhow::Error),
}