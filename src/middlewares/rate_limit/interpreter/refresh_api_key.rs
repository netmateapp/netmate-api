use redis::cmd;

use crate::{common::{api_key::{expiration::ApiKeyExpirationSeconds, key::ApiKey, refreshed_at::LastApiKeyRefreshedAt, API_KEY_EXPIRATION, API_KEY_REFRESH_THERESHOLD}, fallible::Fallible, unixtime::UnixtimeMillis}, helper::redis::{connection::conn, namespace::NAMESPACE_SEPARATOR, namespaces::API_KEY}, middlewares::rate_limit::dsl::refresh_api_key::{ApiKeyRefreshThereshold, RefreshApiKey, RefreshApiKeyError}};

use super::RateLimitImpl;

impl RefreshApiKey for RateLimitImpl {
    fn api_key_refresh_thereshold(&self) -> ApiKeyRefreshThereshold {
        API_KEY_REFRESH_THERESHOLD
    }

    fn api_key_expiration(&self) -> ApiKeyExpirationSeconds {
        API_KEY_EXPIRATION
    }

    async fn refresh_api_key(&self, api_key: &ApiKey, expiration: ApiKeyExpirationSeconds) -> Fallible<(), RefreshApiKeyError> {
        let mut conn = conn(&self.cache, |e| RefreshApiKeyError::RefreshApiKeyFailed(e.into())).await?;
        
        cmd("SET")
            .arg(format!("{}{}{}", API_KEY, NAMESPACE_SEPARATOR, api_key))
            .arg(LastApiKeyRefreshedAt::new(UnixtimeMillis::now()))
            .arg("EX")
            .arg(expiration)
            .exec_async(&mut *conn)
            .await
            .map_err(|e| RefreshApiKeyError::RefreshApiKeyFailed(e.into()))
    }
}