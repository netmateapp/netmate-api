use thiserror::Error;

use crate::common::{fallible::Fallible, id::AccountId, session::value::{RefreshToken, SessionSeries}};

#[derive(Debug, Clone, Copy)]
pub struct RefreshPairExpirationSeconds(u32);

impl RefreshPairExpirationSeconds {
    pub const fn new(seconds: u32) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

impl From<RefreshPairExpirationSeconds> for i32 {
    fn from(expiration: RefreshPairExpirationSeconds) -> Self {
        expiration.0 as i32
    }
}

pub(crate) trait UpdateRefreshToken {
    async fn update_refresh_token(&self, session_series: &SessionSeries, account_id: &AccountId, expiration: &RefreshPairExpirationSeconds) -> Fallible<RefreshToken, UpdateRefreshTokenError> {
        let new_refresh_token = RefreshToken::gen();
        self.assign_new_refresh_token_with_expiration(&new_refresh_token, &session_series, &account_id, expiration).await?;
        Ok(new_refresh_token)
    }

    async fn assign_new_refresh_token_with_expiration(&self, new_refresh_token: &RefreshToken, session_series: &SessionSeries, session_account_id: &AccountId, expiration: &RefreshPairExpirationSeconds) -> Fallible<(), UpdateRefreshTokenError>;
}

#[derive(Debug, Error)]
pub enum UpdateRefreshTokenError {
    #[error("リフレッシュトークの更新に失敗しました")]
    AssignNewRefreshTokenFailed(#[source] anyhow::Error),
}
