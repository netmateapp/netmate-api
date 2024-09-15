use thiserror::Error;

use crate::common::{fallible::Fallible, id::AccountId, session::{refresh_token::RefreshToken, session_id::SessionId, session_series::SessionSeries}};

pub(crate) trait StartSession {
    async fn start_session(&self, session_account_id: &AccountId) -> Fallible<(SessionId, SessionSeries, RefreshToken), StartSessionError>;
}

#[derive(Debug, Error)]
pub enum StartSessionError {
    #[error("セッションの開始に失敗しました")]
    StartSessionFailed(#[source] anyhow::Error),
}