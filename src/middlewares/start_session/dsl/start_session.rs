use http::Response;
use thiserror::Error;

use crate::common::{fallible::Fallible, id::AccountId, session::{refresh_token::RefreshToken, session_id::SessionId, session_series::SessionSeries}};

use super::{assign_refresh_pair::AssignRefreshPair, assign_session_id::AssignSessionId, set_cookie::SetCookie};

pub(crate) trait StartSession {
    async fn start_session<B>(&self, session_account_id: &AccountId, response: Response<B>) -> Fallible<(SessionId, SessionSeries, RefreshToken), StartSessionError>
    where
        Self: AssignSessionId + AssignRefreshPair + SetCookie,
    {

        Err(StartSessionError::StartSessionFailed(anyhow::anyhow!("セッションの開始に失敗しました")))
    }
}

#[derive(Debug, Error)]
pub enum StartSessionError {
    #[error("セッションの開始に失敗しました")]
    StartSessionFailed(#[source] anyhow::Error),
}