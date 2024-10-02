use thiserror::Error;

use crate::{common::{api_key::key::ApiKey, fallible::Fallible}, middlewares::limit::{Count, InculsiveLimit, TimeWindow}};

pub type Rate = Count;

pub(crate) trait IncrementRate {
    async fn try_increment_rate(&self, api_key: &ApiKey) -> Fallible<(), IncrementRateError> {
        let rate = self.increment_rate_within_window(api_key, self.time_window()).await?;
        if self.is_limit_over(rate) {
            return Err(IncrementRateError::RateLimitOver)
        }
        Ok(())
    }

    async fn increment_rate_within_window(&self, api_key: &ApiKey, time_window: TimeWindow) -> Fallible<Rate, IncrementRateError>;

    fn time_window(&self) -> TimeWindow;

    fn is_limit_over(&self, rate: Rate) -> bool {
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

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use crate::{common::{api_key::key::ApiKey, fallible::Fallible}, middlewares::limit::{Count, InculsiveLimit, TimeWindow}};

    use super::{IncrementRate, IncrementRateError, Rate};

    static WITHIN_LIMIT: LazyLock<ApiKey> = LazyLock::new(ApiKey::gen);

    const TIME_WINDOW: TimeWindow = TimeWindow::seconds(60);
    const INCLUSIVE_LIMIT: InculsiveLimit = InculsiveLimit::new(Count::new(100));

    struct MockIncrementRate;

    impl IncrementRate for MockIncrementRate {
        async fn increment_rate_within_window(&self, api_key: &ApiKey, _: TimeWindow) -> Fallible<Rate, IncrementRateError> {
            if api_key == &*WITHIN_LIMIT {
                Ok(Rate::new(INCLUSIVE_LIMIT.value().value()))
            } else {
                Ok(Rate::new(INCLUSIVE_LIMIT.value().value() + 1))
            }
        }

        fn time_window(&self) -> TimeWindow {
            TIME_WINDOW
        }
    
        fn inclusive_limit(&self) -> InculsiveLimit {
            INCLUSIVE_LIMIT
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