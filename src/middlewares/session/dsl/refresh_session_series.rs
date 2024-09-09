use thiserror::Error;

use crate::common::{fallible::Fallible, id::AccountId, session::value::SessionSeries, unixtime::UnixtimeMillis};

use super::update_refresh_token::RefreshPairExpirationSeconds;

pub(crate) trait RefreshSessionSeries {
    async fn try_refresh_session_series(&self, session_series: &SessionSeries, session_account_id: &AccountId, new_expiration: &RefreshPairExpirationSeconds) -> Fallible<(), RefreshSessionSeriesError> {
        let last_refreshed_at = self.fetch_last_session_series_refreshed_at(session_series, session_account_id).await?;
        if Self::should_refresh_session_series(&last_refreshed_at) {
            self.refresh_session_series(session_series, session_account_id, new_expiration).await
        } else {
            Ok(())
        }
    }

    async fn fetch_last_session_series_refreshed_at(&self, session_series: &SessionSeries, session_account_id: &AccountId) -> Fallible<LastSessionSeriesRefreshedTime, RefreshSessionSeriesError>;

    fn should_refresh_session_series(last_refreshed_at: &LastSessionSeriesRefreshedTime) -> bool {
        let now = UnixtimeMillis::now();
        let last_refreshed_at = last_refreshed_at.as_unixtime_millis();
        now.value() - last_refreshed_at.value() >= Self::refresh_thereshold().as_millis()
    }

    fn refresh_thereshold() -> RefreshSessionSeriesThereshold;

    async fn refresh_session_series(&self, session_series: &SessionSeries, session_account_id: &AccountId, new_expiration: &RefreshPairExpirationSeconds) -> Fallible<(), RefreshSessionSeriesError>;
}

#[derive(Debug, Error)]
pub enum RefreshSessionSeriesError {
    #[error("セッションシリーズの更新に失敗しました")]
    UpdateSessionSeriesFailed(#[source] anyhow::Error),
}

pub struct LastSessionSeriesRefreshedTime(UnixtimeMillis);

impl LastSessionSeriesRefreshedTime {
    pub const fn new(time: UnixtimeMillis) -> Self {
        Self(time)
    }

    pub fn as_unixtime_millis(&self) -> &UnixtimeMillis {
        &self.0
    }
}

pub struct RefreshSessionSeriesThereshold(u64);

impl RefreshSessionSeriesThereshold {
    pub const fn days(days: u64) -> Self {
        Self(days)
    }

    pub fn as_millis(&self) -> u64 {
        self.0 * 24 * 60 * 60 * 1000
    }
}