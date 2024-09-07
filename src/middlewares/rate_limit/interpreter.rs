use std::{fs::{self}, sync::Arc};

use redis::{cmd, Script};
use scylla::{frame::value::CqlTimestamp, prepared_statement::PreparedStatement, Session};

use crate::{common::{api_key::ApiKey, fallible::Fallible, unixtime::UnixtimeMillis}, helper::{error::InitError, scylla::prepare, valkey::Pool}};

use super::dsl::{ApiKeyRefreshTimestamp, LimitRateError, RateLimit};

const API_KEY_REFRESH_THERESHOLD: u64 = 10 * 24 * 60 * 60 * 1000;
const CACHE_NAMESPACE: &str = "ratelim";

pub struct RateLimitImpl {
    namespace: &'static str,
    limit: u16,
    interval: u32,
    db: Arc<Session>,
    select_last_api_key_refresh_timestamp: Arc<PreparedStatement>,
    cache: Arc<Pool>,
    incr_if_within_limit: Arc<Script>,
}

impl RateLimitImpl {
    pub async fn try_new(namespace: &'static str, limit: u16, interval: u32, db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<Self>> {
        let select_last_api_key_refresh_timestamp = prepare::<InitError<Self>>(
            &db,
            "SELECT last_refreshed_ttl_at FROM api_keys WHERE key = ?"
        ).await?;

        let lua_script = fs::read_to_string("rate.lua")
            .map_err(|e| InitError::new(e.into()))?;
        let incr_if_within_limit = Arc::new(Script::new(lua_script.as_str()));

        Ok(Self { namespace, limit, interval, db, select_last_api_key_refresh_timestamp, cache, incr_if_within_limit })
    }
}

impl RateLimit for RateLimitImpl {
    async fn increment_rate(&self, api_key: &ApiKey) -> Fallible<u16, LimitRateError> {
        let mut conn = self.cache
            .get()
            .await
            .map_err(|e| LimitRateError::IncrementRateFailed(e.into()))?;

        self.incr_if_within_limit
                .key(format!("{}:{}:{}", CACHE_NAMESPACE, self.namespace, api_key.value().value()))
                .arg(self.interval)
                .invoke_async::<u16>(&mut *conn)
                .await
                .map_err(|e| LimitRateError::IncrementRateFailed(e.into()))
    }

    fn limit(&self) -> u16 {
        self.limit
    }

    // ScyllaDBのキャッシュは高速であるため問題ないが、
    // 複数のエンドポイントで同じ検証をするのは効率が悪いので、
    // 30分～1時間程度の短時間キャッシュを行うべき(リフレッシュ時刻も併せてキャッシュするため、短時間にする必要がある)
    async fn check_api_key_exists(&self, api_key: &ApiKey) -> Fallible<Option<ApiKeyRefreshTimestamp>, LimitRateError> {
        self.db
            .execute(&self.select_last_api_key_refresh_timestamp, (api_key.value().value(),))
            .await
            .map_err(|e| LimitRateError::CheckApiKeyExistsFailed(e.into()))?
            .first_row_typed::<(CqlTimestamp, )>()
            .map(|(last_refreshed_ttl_at, )| Some(ApiKeyRefreshTimestamp::new(UnixtimeMillis::new(last_refreshed_ttl_at.0 as u64))))
            .map_err(|e| LimitRateError::CheckApiKeyExistsFailed(e.into()))
    }

    fn should_refresh_api_key(&self, api_key_refresh_timestamp: &ApiKeyRefreshTimestamp) -> bool {
        UnixtimeMillis::now().value() - api_key_refresh_timestamp.value().value() > API_KEY_REFRESH_THERESHOLD
    }

    async fn refresh_api_key(&self, api_key: &ApiKey) -> Fallible<(), LimitRateError> {
        self.db
            .execute(&self.select_last_api_key_refresh_timestamp, (api_key.value().value(),))
            .await
            .map(|_| ())
            .map_err(|e| LimitRateError::RefreshApiKeyFailed(e.into()))
    }
}