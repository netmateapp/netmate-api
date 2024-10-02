use redis::cmd;

use crate::{common::{api_key::{key::ApiKey, refreshed_at::LastApiKeyRefreshedAt}, fallible::Fallible}, helper::redis::{connection::conn, namespace::NAMESPACE_SEPARATOR, namespaces::API_KEY}, middlewares::rate_limit::dsl::rate_limit::{RateLimit, RateLimitError}};

use super::RateLimitImpl;

impl RateLimit for RateLimitImpl {
    // 複数のエンドポイントで同じ検証をするのは効率が悪いので、
    // 30分～1時間程度の短時間キャッシュを行うべき(リフレッシュ時刻も併せてキャッシュするため、短時間にする必要がある)
    async fn fetch_last_api_key_refreshed_at(&self, api_key: &ApiKey) -> Fallible<Option<LastApiKeyRefreshedAt>, RateLimitError> {
        let mut conn = conn(&self.cache, |e| RateLimitError::FetchLastApiKeyRefreshedAt(e.into())).await?;
        
        cmd("GET")
            .arg(format!("{}{}{}", API_KEY, NAMESPACE_SEPARATOR, api_key))
            .query_async::<Option<LastApiKeyRefreshedAt>>(&mut *conn)
            .await
            .map_err(|e| RateLimitError::FetchLastApiKeyRefreshedAt(e.into()))
    }
}