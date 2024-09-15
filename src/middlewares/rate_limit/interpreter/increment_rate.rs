
use redis::{Script, ToRedisArgs};

use crate::{common::{api_key::ApiKey, fallible::Fallible}, helper::redis::{Connection, TypedCommand, NAMESPACE_SEPARATOR}, middlewares::rate_limit::{dsl::increment_rate::{IncrementRate, IncrementRateError, InculsiveLimit, Rate, TimeWindow}, interpreter::RATE_LIMIT_NAMESPACE}};

use super::{EndpointName, RateLimitImpl};

impl IncrementRate for RateLimitImpl {
    async fn increment_rate_within_window(&self, api_key: &ApiKey, time_window: &TimeWindow) -> Fallible<Rate, IncrementRateError> {
        let key = Key(&self.endpoint_name, api_key);
        
        self.incr_and_expire_if_first
                .run(&self.cache, (key, time_window))
                .await
                .map_err(IncrementRateError::IncrementRateFailed)
    }

    fn time_window(&self) -> &TimeWindow {
        &self.time_window
    }

    fn inclusive_limit(&self) -> &InculsiveLimit {
        &self.limit
    }
}

#[derive(Debug)]
pub struct IncrAndExpireIfFirstScript(pub Script);

struct Key<'a, 'b>(&'a EndpointName, &'b ApiKey);

fn format_key(endpoint_name: &EndpointName, api_key: &ApiKey) -> String {
    format!("{}{}{}{}{}", RATE_LIMIT_NAMESPACE, NAMESPACE_SEPARATOR, endpoint_name, NAMESPACE_SEPARATOR, api_key)
}

impl<'a, 'b> ToRedisArgs for Key<'a, 'b> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        format_key(self.0, self.1).write_redis_args(out);
    }
}

impl<'a, 'b, 'c> TypedCommand<(Key<'a, 'b>, &'c TimeWindow), Rate> for IncrAndExpireIfFirstScript {
    async fn execute(&self, mut conn: Connection<'_>, (key, time_window): (Key<'a, 'b>, &'c TimeWindow)) -> anyhow::Result<Rate> {
        self.0
            .key(key)
            .arg(time_window)
            .invoke_async::<Rate>(&mut *conn)
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use crate::{common::api_key::ApiKey, helper::redis::{Namespace, NAMESPACE_SEPARATOR}, middlewares::rate_limit::interpreter::{EndpointName, RATE_LIMIT_NAMESPACE}};

    use super::format_key;

    #[test]
    fn test_format_key() {
        let endpoint_name = EndpointName::new(Namespace::new("test").unwrap());
        let api_key = ApiKey::gen();
        let key = format_key(&endpoint_name, &api_key);
        let expected = format!("{}{}{}{}{}", RATE_LIMIT_NAMESPACE, NAMESPACE_SEPARATOR, endpoint_name, NAMESPACE_SEPARATOR, api_key);
        assert_eq!(key, expected);
    }
}