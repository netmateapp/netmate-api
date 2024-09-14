use crate::{common::{api_key::ApiKey, fallible::Fallible}, helper::valkey::conn, middlewares::rate_limit::{dsl::increment_rate::{IncrementRate, IncrementRateError, InculsiveLimit, Rate, TimeWindow}, interpreter::BASE_NAMESPACE}};

use super::RateLimitImpl;

impl IncrementRate for RateLimitImpl {
    async fn increment_rate_within_window(&self, api_key: &ApiKey, time_window: &TimeWindow) -> Fallible<Rate, IncrementRateError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> IncrementRateError {
            IncrementRateError::IncrementRateFailed(e.into())
        }
        
        let mut conn = conn(&self.cache, handle_error).await?;

        let key = format!("{}:{}:{}", BASE_NAMESPACE, self.endpoint_name.value(), api_key.value().value());

        self.incr_and_expire_if_first
                .key(key)
                .arg(time_window.as_secs())
                .invoke_async::<u32>(&mut *conn)
                .await
                .map(Rate::new)
                .map_err(handle_error)
    }

    fn time_window(&self) -> &TimeWindow {
        &self.time_window
    }

    fn inclusive_limit(&self) -> &InculsiveLimit {
        &self.limit
    }
}