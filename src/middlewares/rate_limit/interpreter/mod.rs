use std::sync::Arc;

use redis::Script;
use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{helper::{error::InitError, redis::{namespace::Namespace, Pool}, scylla::prepare}, middlewares::limit::{EndpointName, InculsiveLimit, TimeWindow}};

mod increment_rate;
mod rate_limit;
mod refresh_api_key;

const RATE_LIMIT_NAMESPACE: Namespace = Namespace::of("rtlim");

#[derive(Debug)]
pub struct RateLimitImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    endpoint_name: EndpointName,
    limit: InculsiveLimit,
    time_window: TimeWindow,
    select_last_api_key_refreshed_at: Arc<PreparedStatement>,
    insert_api_key_with_ttl_refresh: Arc<PreparedStatement>,
    incr_and_expire_if_first: Arc<Script>,
}

impl RateLimitImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>, endpoint_name: EndpointName, limit: InculsiveLimit, time_window: TimeWindow) -> Result<Self, InitError<Self>> {
        let select_last_api_key_refreshed_at = prepare(&db, "SELECT refreshed_at FROM api_keys WHERE api_key = ?").await?;

        let insert_api_key_with_ttl_refresh = prepare(&db, "INSERT INTO api_keys (api_key, refreshed_at) VALUES (?, ?) USING TTL ?").await?;

        let incr_and_expire_if_first = Arc::new(Script::new(include_str!("incr_and_expire_if_first.lua")));

        Ok(Self { endpoint_name, limit, time_window, db, select_last_api_key_refreshed_at, insert_api_key_with_ttl_refresh, cache, incr_and_expire_if_first })
    }
}