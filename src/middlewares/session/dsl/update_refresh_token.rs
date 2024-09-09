use crate::common::{fallible::Fallible, id::AccountId, session::value::{RefreshToken, SessionSeries}};

pub struct RefreshTokenExpirationSeconds(u32);

impl RefreshTokenExpirationSeconds {
    pub const fn new(seconds: u32) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

pub(crate) trait UpdateRefreshToken {
    async fn update_refresh_token(&self, session_series: &SessionSeries, account_id: &AccountId, expiration: &RefreshTokenExpirationSeconds) -> Fallible<RefreshToken, UpdateRefreshTokenError> {
        let new_refresh_token = RefreshToken::gen();
        self.active_new_refresh_token_with_expiration(&new_refresh_token, &session_series, &account_id, expiration).await?;
        Ok(new_refresh_token)
    }

    async fn active_new_refresh_token_with_expiration(&self, new_refresh_token: &RefreshToken, session_series: &SessionSeries, session_account_id: &AccountId, expiration: &RefreshTokenExpirationSeconds) -> Fallible<(), UpdateRefreshTokenError>;
}

pub enum UpdateRefreshTokenError {
    IssueNewRefreshTokenFailed,
}
