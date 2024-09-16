use redis::{FromRedisValue, RedisResult, ToRedisArgs};
use thiserror::Error;

use crate::common::{api_key::ApiKey, fallible::Fallible};

pub(crate) trait IncrementRate {
    async fn try_increment_rate(&self, api_key: &ApiKey) -> Fallible<(), IncrementRateError> {
        let rate = self.increment_rate_within_window(api_key, self.time_window()).await?;
        if self.is_limit_over(&rate) {
            return Err(IncrementRateError::RateLimitOver)
        }
        Ok(())
    }

    async fn increment_rate_within_window(&self, api_key: &ApiKey, time_window: &TimeWindow) -> Fallible<Rate, IncrementRateError>;

    fn time_window(&self) -> &TimeWindow;

    fn is_limit_over(&self, rate: &Rate) -> bool {
        rate > self.inclusive_limit().value()
    }

    fn inclusive_limit(&self) -> &InculsiveLimit;
}

#[derive(Debug, Error)]
pub enum IncrementRateError {
    #[error("レートの取得に失敗しました")]
    IncrementRateFailed(#[source] anyhow::Error),
    #[error("レート上限に達しています")]
    RateLimitOver,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TimeWindow(u32);

impl TimeWindow {
    pub const fn seconds(seconds: u32) -> Self {
        Self(seconds)
    }

    pub const fn minutes(minutes: u32) -> Self {
        Self::seconds(minutes * 60)
    }

    pub const fn hours(hours: u32) -> Self {
        Self::minutes(hours * 60)
    }

    pub const fn days(days: u32) -> Self {
        Self::hours(days * 24)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

impl ToRedisArgs for TimeWindow {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        self.0.write_redis_args(out);
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Rate(u32);

impl Rate {
    pub const fn new(rate: u32) -> Self {
        Self(rate)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl FromRedisValue for Rate {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        u32::from_redis_value(v).map(Rate)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct InculsiveLimit(Rate);

impl InculsiveLimit {
    pub const fn new(limit: u32) -> Self {
        Self(Rate::new(limit))
    }

    pub fn value(&self) -> &Rate {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use crate::common::{api_key::ApiKey, fallible::Fallible};

    use super::{IncrementRate, IncrementRateError, InculsiveLimit, Rate, TimeWindow};

    static WITHIN_LIMIT: LazyLock<ApiKey> = LazyLock::new(ApiKey::gen);

    const TIME_WINDOW: TimeWindow = TimeWindow::seconds(60);
    const INCLUSIVE_LIMIT: InculsiveLimit = InculsiveLimit::new(100);

    struct MockIncrementRate;

    impl IncrementRate for MockIncrementRate {
        async fn increment_rate_within_window(&self, api_key: &ApiKey, _: &TimeWindow) -> Fallible<Rate, IncrementRateError> {
            if api_key == &*WITHIN_LIMIT {
                Ok(Rate::new(INCLUSIVE_LIMIT.value().value()))
            } else {
                Ok(Rate::new(INCLUSIVE_LIMIT.value().value() + 1))
            }
        }

        fn time_window(&self) -> &TimeWindow {
            &TIME_WINDOW
        }
    
        fn inclusive_limit(&self) -> &InculsiveLimit {
            &INCLUSIVE_LIMIT
        }
    }

    #[tokio::test]
    async fn within_limit() {
        let api_key = &*WITHIN_LIMIT;
        let result = MockIncrementRate.try_increment_rate(api_key).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn over_limit() {
        let api_key = ApiKey::gen();
        let result = MockIncrementRate.try_increment_rate(&api_key).await;
        match result {
            Err(IncrementRateError::RateLimitOver) => (),
            _ => panic!(),
        }
    }
}