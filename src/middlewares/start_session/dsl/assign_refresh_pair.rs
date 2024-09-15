use thiserror::Error;

use crate::common::{fallible::Fallible, id::account_id::AccountId, session::{refresh_pair_expiration::RefreshPairExpirationSeconds, refresh_token::RefreshToken, session_series::SessionSeries}};

pub(crate)  trait AssignRefreshPair {
    async fn assign_refresh_pair(&self, session_account_id: AccountId, expiration: RefreshPairExpirationSeconds) -> Fallible<(SessionSeries, RefreshToken), AssignRefreshPairError> {
        let mut session_series = SessionSeries::gen();
        let refresh_token = RefreshToken::gen();

        // このループは奇跡が起きない限りO(1)となる
        loop {
            match self.try_assign_refresh_pair_with_expiration_if_unused(&session_series, &refresh_token, session_account_id, expiration).await {
                Ok(()) => return Ok((session_series, refresh_token)),
                Err(AssignRefreshPairError::SessionIdAlreadyUsed) => session_series = SessionSeries::gen(),
                Err(e) => return Err(e),
            }
        }
    }

    async fn try_assign_refresh_pair_with_expiration_if_unused(&self, session_series: &SessionSeries, refresh_token: &RefreshToken, session_account_id: AccountId, expiration: RefreshPairExpirationSeconds) -> Fallible<(), AssignRefreshPairError>;
}

#[derive(Debug, Error)]
pub enum AssignRefreshPairError {
    #[error("セッションIDが既に使用されています")]
    SessionIdAlreadyUsed,
    #[error("新規セッションIDの割り当てに失敗しました")]
    AssignNewSessionIdFailed(#[source] anyhow::Error),
}
