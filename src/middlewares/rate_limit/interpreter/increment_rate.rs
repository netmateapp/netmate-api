
use redis::{RedisWrite, ToRedisArgs};

use crate::{common::{api_key::ApiKey, fallible::Fallible}, helper::redis::{conn, NAMESPACE_SEPARATOR}, middlewares::rate_limit::{dsl::increment_rate::{IncrementRate, IncrementRateError, InculsiveLimit, Rate, TimeWindow}, interpreter::RATE_LIMIT_NAMESPACE}};

use super::{EndpointName, RateLimitImpl};

impl IncrementRate for RateLimitImpl {
    async fn increment_rate_within_window(&self, api_key: &ApiKey, time_window: &TimeWindow) -> Fallible<Rate, IncrementRateError> {
        let mut conn = conn(&self.cache, |e| IncrementRateError::IncrementRateFailed(e.into())).await?;
        
        self.incr_and_expire_if_first
            .key(RateKey::new(&self.endpoint_name, api_key))
            .arg(time_window)
            .invoke_async::<Rate>(&mut *conn)
            .await
            .map_err(|e| IncrementRateError::IncrementRateFailed(e.into()))
    }

    fn time_window(&self) -> &TimeWindow {
        &self.time_window
    }

    fn inclusive_limit(&self) -> &InculsiveLimit {
        &self.limit
    }
}

struct RateKey(String);

impl RateKey {
    pub fn new(endpoint_name: &EndpointName, api_key: &ApiKey) -> Self {
        Self(format!("{}{}{}{}{}", RATE_LIMIT_NAMESPACE, NAMESPACE_SEPARATOR, endpoint_name, NAMESPACE_SEPARATOR, api_key))
    }
}

impl ToRedisArgs for RateKey {
    fn write_redis_args<W: ?Sized + RedisWrite>(&self, out: &mut W) {
        self.0.write_redis_args(out);
    }
}

#[cfg(test)]
mod tests {
    use crate::{common::api_key::ApiKey, helper::redis::{Namespace, NAMESPACE_SEPARATOR}, middlewares::rate_limit::interpreter::{increment_rate::RateKey, EndpointName, RATE_LIMIT_NAMESPACE}};

    #[test]
    fn test_format_key() {
        let endpoint_name = EndpointName::new(Namespace::new("test").unwrap());
        let api_key = ApiKey::gen();
        let key = RateKey::new(&endpoint_name, &api_key);
        let expected = format!("{}{}{}{}{}", RATE_LIMIT_NAMESPACE, NAMESPACE_SEPARATOR, endpoint_name, NAMESPACE_SEPARATOR, api_key);
        assert_eq!(key.0, expected);
    }
}