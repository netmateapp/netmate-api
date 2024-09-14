use std::{fmt::{self, Display}, sync::{Arc, LazyLock}};

use rate_limit::{SelectLastApiKeyRefreshedAt, SELECT_LAST_API_KEY_REFRESHED_AT};
use redis::Script;
use refresh_api_key::{InsertApiKeyWithTtlRefresh, INSERT_API_KEY_WITH_TTL_REFRESH};
use scylla::Session;

use crate::{helper::{error::InitError, redis::{Namespace, Pool}, scylla::prepare}, middlewares::rate_limit::dsl::increment_rate::{InculsiveLimit, TimeWindow}};

mod increment_rate;
mod rate_limit;
mod refresh_api_key;

const BASE_NAMESPACE: Namespace = Namespace::of("rtlim");

#[derive(Debug)]
pub struct RateLimitImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    endpoint_name: EndpointName,
    limit: InculsiveLimit,
    time_window: TimeWindow,
    select_last_api_key_refreshed_at: Arc<SelectLastApiKeyRefreshedAt>,
    insert_api_key_with_ttl_refresh: Arc<InsertApiKeyWithTtlRefresh>,
    incr_and_expire_if_first: Arc<Script>,
}

impl RateLimitImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>, endpoint_name: EndpointName, limit: InculsiveLimit, time_window: TimeWindow) -> Result<Self, InitError<Self>> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> InitError<RateLimitImpl> {
            InitError::new(e.into())
        }

        let select_last_api_key_refreshed_at = prepare(&db, SelectLastApiKeyRefreshedAt, SELECT_LAST_API_KEY_REFRESHED_AT)
            .await
            .map_err(handle_error)?;

        let insert_api_key_with_ttl_refresh = prepare(&db, InsertApiKeyWithTtlRefresh, INSERT_API_KEY_WITH_TTL_REFRESH)
            .await
            .map_err(handle_error)?;

        let incr_and_expire_if_first = Arc::new(
            Script::new(include_str!("incr_and_expire_if_first.lua"))
        );

        Ok(Self { endpoint_name, limit, time_window, db, select_last_api_key_refreshed_at, insert_api_key_with_ttl_refresh, cache, incr_and_expire_if_first })
    }
}

#[derive(Debug)]
pub struct EndpointName(Namespace);

impl From<Namespace> for EndpointName {
    fn from(namespace: Namespace) -> Self {
        Self(namespace)
    }
}

impl Display for EndpointName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}