use crate::{common::{api_key::ApiKey, fallible::Fallible}, middlewares::rate_limit::dsl::rate_limit::{LastApiKeyRefreshedAt, RateLimit, RateLimitError}};

use super::RateLimitImpl;

impl RateLimit for RateLimitImpl {
    // ScyllaDBのキャッシュは高速であるため問題ないが、
    // 複数のエンドポイントで同じ検証をするのは効率が悪いので、
    // 30分～1時間程度の短時間キャッシュを行うべき(リフレッシュ時刻も併せてキャッシュするため、短時間にする必要がある)
    async fn fetch_last_api_key_refreshed_at(&self, api_key: &ApiKey) -> Fallible<Option<LastApiKeyRefreshedAt>, RateLimitError> {
        self.db
            .execute_unpaged(&self.select_last_api_key_refreshed_at, (api_key, ))
            .await
            .map_err(|e| RateLimitError::FetchLastApiKeyRefreshedAt(e.into()))?
            .maybe_first_row_typed::<(LastApiKeyRefreshedAt, )>()
            .map(|o| o.map(|(refreshed_at, )| refreshed_at))
            .map_err(|e| RateLimitError::FetchLastApiKeyRefreshedAt(e.into()))
    }
}