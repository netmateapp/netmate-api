use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use thiserror::Error;

use crate::common::{api_key::ApiKey, fallible::Fallible, unixtime::UnixtimeMillis};

use super::rate_limit::LastApiKeyRefreshedAt;

pub(crate) trait RefreshApiKey {
    async fn try_refresh_api_key(&self, last_api_key_refreshed_at: &LastApiKeyRefreshedAt, api_key: &ApiKey) -> Fallible<(), RefreshApiKeyError> {
        if self.should_refresh_api_key(&last_api_key_refreshed_at) {
            self.refresh_api_key(api_key, self.api_key_expiration()).await
        } else {
            Err(RefreshApiKeyError::NoNeedToRefreshApiKey)
        }
    }

    fn should_refresh_api_key(&self, last_api_key_refreshed_at: &LastApiKeyRefreshedAt) -> bool {
        let now = UnixtimeMillis::now();
        let last_refreshed_at = last_api_key_refreshed_at.value();
        now.value() - last_refreshed_at.value() >= self.api_key_refresh_thereshold().as_millis()
    }

    fn api_key_refresh_thereshold(&self) -> &ApiKeyRefreshThereshold;

    fn api_key_expiration(&self) -> &ApiKeyExpirationSeconds;

    async fn refresh_api_key(&self, api_key: &ApiKey, expiration: &ApiKeyExpirationSeconds) -> Fallible<(), RefreshApiKeyError>;
}

#[derive(Debug, Error)]
pub enum RefreshApiKeyError {
    #[error("APIキーの更新の必要がありません")]
    NoNeedToRefreshApiKey,
    #[error("APIキーの更新に失敗しました")]
    RefreshApiKeyFailed(#[source] anyhow::Error),
}

pub struct ApiKeyRefreshThereshold(u64);

impl ApiKeyRefreshThereshold {
    pub const fn days(days: u64) -> Self {
        Self(days)
    }

    pub fn as_millis(&self) -> u64 {
        self.0 * 24 * 60 * 60 * 1000
    }
}

#[derive(Debug, Clone)]
pub struct ApiKeyExpirationSeconds(u64);

impl ApiKeyExpirationSeconds {
    pub const fn secs(seconds: u64) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u64 {
        self.0
    }
}

impl From<ApiKeyExpirationSeconds> for i64 {
    fn from(expiration: ApiKeyExpirationSeconds) -> i64 {
        expiration.0 as i64
    }
}

impl SerializeValue for ApiKeyExpirationSeconds {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        (self.0 as i64).serialize(typ, writer)
    }
}

#[cfg(test)]
mod tests {
    use crate::{common::{api_key::ApiKey, fallible::Fallible, unixtime::UnixtimeMillis}, middlewares::rate_limit::dsl::rate_limit::LastApiKeyRefreshedAt};

    use super::{ApiKeyExpirationSeconds, ApiKeyRefreshThereshold, RefreshApiKey, RefreshApiKeyError};

    const API_KEY_REFRESH_THERESHOLD: ApiKeyRefreshThereshold = ApiKeyRefreshThereshold::days(1);
    const API_KEY_EXPIRATION: ApiKeyExpirationSeconds = ApiKeyExpirationSeconds::secs(60);

    struct MockRefreshApiKey;

    impl RefreshApiKey for MockRefreshApiKey {
        fn api_key_refresh_thereshold(&self) -> &ApiKeyRefreshThereshold {
            &API_KEY_REFRESH_THERESHOLD
        }

        fn api_key_expiration(&self) -> &ApiKeyExpirationSeconds {
            &API_KEY_EXPIRATION
        }

        async fn refresh_api_key(&self, _api_key: &ApiKey, _expiration: &ApiKeyExpirationSeconds) -> Fallible<(), RefreshApiKeyError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn api_key_to_be_refreshed() {
        let last_api_key_refreshed_at = UnixtimeMillis::now().value() - API_KEY_REFRESH_THERESHOLD.as_millis();
        let last_api_key_refreshed_at = LastApiKeyRefreshedAt::new(UnixtimeMillis::new(last_api_key_refreshed_at));
        let api_key = ApiKey::gen();
        let result = MockRefreshApiKey.try_refresh_api_key(&last_api_key_refreshed_at, &api_key).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn api_key_not_to_be_refreshed() {
        let last_api_key_refreshed_at = LastApiKeyRefreshedAt::new(UnixtimeMillis::now());
        let api_key = ApiKey::gen();
        let result = MockRefreshApiKey.try_refresh_api_key(&last_api_key_refreshed_at, &api_key).await;
        assert!(result.is_err());
    }
}