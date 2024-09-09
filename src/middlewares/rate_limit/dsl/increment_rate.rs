use thiserror::Error;

use crate::common::{api_key::ApiKey, fallible::Fallible};

pub(crate) trait IncrementRate {
    async fn try_increment_rate(&self, api_key: &ApiKey) -> Fallible<(), IncrementRateError> {
        let rate = self.increment_rate_within_window(api_key, &self.time_window()).await?;
        if self.is_limit_over(&rate) {
            return Err(IncrementRateError::RateLimitOver)
        }
        Ok(())
    }

    async fn increment_rate_within_window(&self, api_key: &ApiKey, window: &TimeWindow) -> Fallible<Rate, IncrementRateError>;

    fn time_window(&self) -> TimeWindow;

    fn is_limit_over(&self, rate: &Rate) -> bool {
        rate > self.inclusive_limit().value()
    }

    fn inclusive_limit(&self) -> InculsiveLimit;
}

#[derive(Debug, Error)]
pub enum IncrementRateError {
    #[error("レートの取得に失敗しました")]
    IncrementRateFailed(#[source] anyhow::Error),
    #[error("レート上限に達しています")]
    RateLimitOver,
}

pub struct TimeWindow(u32);

impl TimeWindow {
    pub fn secs(seconds: u32) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rate(u32);

impl Rate {
    pub fn new(rate: u32) -> Self {
        Self(rate)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

pub struct InculsiveLimit(Rate);

impl InculsiveLimit {
    pub fn new(limit: u32) -> Self {
        Self(Rate(limit))
    }

    pub fn value(&self) -> &Rate {
        &self.0
    }
}