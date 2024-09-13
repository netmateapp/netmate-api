use std::sync::Arc;

use redis::Script;
use scylla::{prepared_statement::PreparedStatement, FromRow, Session};
use thiserror::Error;

use crate::{common::{api_key::ApiKey, fallible::Fallible, unixtime::UnixtimeMillis}, helper::{error::InitError, scylla::{prepare, Statement, TypedStatement, Unit}, valkey::{conn, Pool}}, middlewares::rate_limit::dsl::{increment_rate::{IncrementRate, IncrementRateError, InculsiveLimit, Rate, TimeWindow}, rate_limit::{LastApiKeyRefreshedAt, RateLimit, RateLimitError}, refresh_api_key::{ApiKeyExpirationSeconds, ApiKeyRefreshThereshold, RefreshApiKey, RefreshApiKeyError}}};

const BASE_NAMESPACE: &str = "rtlim";

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
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>, namespace: EndpointName, limit: InculsiveLimit, time_window: TimeWindow) -> Result<Self, InitError<Self>> {
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

        Ok(Self { endpoint_name: namespace, limit, time_window, db, select_last_api_key_refreshed_at, insert_api_key_with_ttl_refresh, cache, incr_and_expire_if_first })
    }
}

impl RateLimit for RateLimitImpl {
    // ScyllaDBのキャッシュは高速であるため問題ないが、
    // 複数のエンドポイントで同じ検証をするのは効率が悪いので、
    // 30分～1時間程度の短時間キャッシュを行うべき(リフレッシュ時刻も併せてキャッシュするため、短時間にする必要がある)
    async fn fetch_last_api_key_refreshed_at(&self, api_key: &ApiKey) -> Fallible<Option<LastApiKeyRefreshedAt>, RateLimitError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> RateLimitError {
            RateLimitError::FetchLastApiKeyRefreshedAt(e.into())
        }

        self.select_last_api_key_refreshed_at
            .query(&self.db, (api_key, ))
            .await
            .map(|o| o.map(|(refreshed_at, )| refreshed_at))
            .map_err(handle_error)
    }
}

const SELECT_LAST_API_KEY_REFRESHED_AT: Statement<SelectLastApiKeyRefreshedAt> = Statement::of("SELECT refreshed_at FROM api_keys WHERE api_key = ?");

#[derive(Debug)]
struct SelectLastApiKeyRefreshedAt(PreparedStatement);

impl<'a> TypedStatement<(&'a ApiKey, ), (LastApiKeyRefreshedAt, )> for SelectLastApiKeyRefreshedAt {
    type Result<U> = Option<U> where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (&'a ApiKey, )) -> anyhow::Result<Self::Result<(LastApiKeyRefreshedAt, )>> {
        session.execute_unpaged(&self.0, values)
            .await
            .map_err(anyhow::Error::from)?
            .maybe_first_row_typed()
            .map_err(anyhow::Error::from)
    }
}

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

const API_KEY_REFRESH_THERESHOLD: ApiKeyRefreshThereshold = ApiKeyRefreshThereshold::days(10);
const API_KEY_EXPIRATION: ApiKeyExpirationSeconds = ApiKeyExpirationSeconds::secs(2592000);

impl RefreshApiKey for RateLimitImpl {
    fn api_key_refresh_thereshold(&self) -> &ApiKeyRefreshThereshold {
        &API_KEY_REFRESH_THERESHOLD
    }

    fn api_key_expiration(&self) -> &ApiKeyExpirationSeconds {
        &API_KEY_EXPIRATION
    }

    async fn refresh_api_key(&self, api_key: &ApiKey, expiration: &ApiKeyExpirationSeconds) -> Fallible<(), RefreshApiKeyError> {
        self.insert_api_key_with_ttl_refresh
            .execute(&self.db, (api_key, &UnixtimeMillis::now(), expiration))
            .await
            .map_err(|e| RefreshApiKeyError::RefreshApiKeyFailed(e.into()))
    }
}

const INSERT_API_KEY_WITH_TTL_REFRESH: Statement<InsertApiKeyWithTtlRefresh> = Statement::of("INSERT INTO api_keys (api_key, refreshed_at) VALUES (?, ?) USING TTL ?");

#[derive(Debug)]
struct InsertApiKeyWithTtlRefresh(PreparedStatement);

impl<'a> TypedStatement<(&'a ApiKey, &'a UnixtimeMillis, &'a ApiKeyExpirationSeconds), Unit> for InsertApiKeyWithTtlRefresh {
    type Result<U> = U where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (&'a ApiKey, &'a UnixtimeMillis, &'a ApiKeyExpirationSeconds)) -> anyhow::Result<Unit> {
        session.execute_unpaged(&self.0, values)
            .await
            .map(|_| Unit)
            .map_err(anyhow::Error::from)
    }
}

const MIN_NAMESPACE_LENGTH: usize = 3;
const MAX_NAMESPACE_LENGTH: usize = 9;

#[derive(Debug)]
pub struct EndpointName(&'static str);

impl EndpointName {
    pub fn new(endpoint_name: &'static str) -> Result<Self, ParseEndpointNameError> {
        if endpoint_name.contains(':') {
            Err(ParseEndpointNameError::ContainsColon)
        } else if !endpoint_name.is_ascii() {
            Err(ParseEndpointNameError::NotAscii)
        } else if endpoint_name.len() < MIN_NAMESPACE_LENGTH {
            Err(ParseEndpointNameError::TooShort)
        } else if endpoint_name.len() > MAX_NAMESPACE_LENGTH {
            Err(ParseEndpointNameError::TooLong)
        } else {
            Ok(Self(endpoint_name))
        }
    }

    pub fn value(&self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Error)]
pub enum ParseEndpointNameError {
    #[error("コロンは許可されていません")]
    ContainsColon,
    #[error("ASCII文字列である必要があります")]
    NotAscii,
    #[error("{}文字以上である必要があります", MIN_NAMESPACE_LENGTH)]
    TooShort,
    #[error("{}文字以下である必要があります", MAX_NAMESPACE_LENGTH)]
    TooLong
}