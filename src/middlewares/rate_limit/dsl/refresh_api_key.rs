use thiserror::Error;

use crate::common::{api_key::ApiKey, fallible::Fallible, unixtime::UnixtimeMillis};

use super::rate_limit::LastApiKeyRefreshedAt;

pub(crate) trait RefreshApiKey {
    async fn try_refresh_api_key(&self, last_api_key_refreshed_at: &LastApiKeyRefreshedAt, api_key: &ApiKey) -> Fallible<(), RefreshApiKeyError> {
        if self.should_refresh_api_key(&last_api_key_refreshed_at) {
            self.refresh_api_key(api_key).await
        } else {
            Err(RefreshApiKeyError::NoNeedToRefreshApiKey)
        }
    }

    fn should_refresh_api_key(&self, last_api_key_refreshed_at: &LastApiKeyRefreshedAt) -> bool {
        let now = UnixtimeMillis::now();
        let last_refreshed_at = last_api_key_refreshed_at.value();
        now.value() - last_refreshed_at.value() >= Self::refresh_thereshold().as_millis()
    }

    fn refresh_thereshold() -> RefreshApiKeyThereshold;

    async fn refresh_api_key(&self, api_key: &ApiKey) -> Fallible<(), RefreshApiKeyError>;
}

#[derive(Debug, Error)]
pub enum RefreshApiKeyError {
    #[error("APIキーの更新の必要がありません")]
    NoNeedToRefreshApiKey,
    #[error("APIキーの更新に失敗しました")]
    RefreshApiKeyFailed(#[source] anyhow::Error),
}

pub struct RefreshApiKeyThereshold(u64);

impl RefreshApiKeyThereshold {
    pub const fn days(days: u64) -> Self {
        Self(days)
    }

    pub fn as_millis(&self) -> u64 {
        self.0 * 24 * 60 * 60 * 1000
    }
}

#[cfg(test)]
mod tests {
    use crate::{common::{api_key::ApiKey, fallible::Fallible, unixtime::UnixtimeMillis}, middlewares::rate_limit::dsl::rate_limit::LastApiKeyRefreshedAt};

    use super::{RefreshApiKey, RefreshApiKeyError, RefreshApiKeyThereshold};

    const REFRESH_API_KEY_THERESHOLD: RefreshApiKeyThereshold = RefreshApiKeyThereshold::days(1);

    struct MockRefreshApiKey;

    impl RefreshApiKey for MockRefreshApiKey {
        fn refresh_thereshold() -> RefreshApiKeyThereshold {
            REFRESH_API_KEY_THERESHOLD
        }

        async fn refresh_api_key(&self, _api_key: &super::ApiKey) -> Fallible<(), RefreshApiKeyError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn api_key_to_be_refreshed() {
        let last_api_key_refreshed_at = UnixtimeMillis::now().value() - REFRESH_API_KEY_THERESHOLD.as_millis();
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