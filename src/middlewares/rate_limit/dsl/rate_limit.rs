use std::{convert::Infallible, str::FromStr};

use http::{HeaderMap, Request, Response};
use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::CqlValue};
use thiserror::Error;
use tower::Service;

use crate::common::{api_key::ApiKey, fallible::Fallible, unixtime::UnixtimeMillis};

use super::{increment_rate::{IncrementRate, IncrementRateError}, refresh_api_key::RefreshApiKey};

pub(crate) trait RateLimit {
    async fn rate_limit<S, B>(&self, inner: &mut S, request: Request<B>) -> Fallible<S::Response, RateLimitError>
    where
        Self: IncrementRate + RefreshApiKey,
        S: Service<Request<B>, Error = Infallible, Response = Response<B>>
    {
        match Self::extract_no_account_user_api_key(request.headers()) {
            Some(api_key) => {
                match self.fetch_last_api_key_refreshed_at(&api_key).await? {
                    Some(last_api_key_refreshed_at) => {
                        match self.try_increment_rate(&api_key).await {
                            Ok(_) => {
                                // `Error`は`Infallible`であるため`unwrap()`で問題ない
                                let response = inner.call(request).await.unwrap();

                                let _ = self.try_refresh_api_key(last_api_key_refreshed_at, &api_key).await;

                                Ok(response)
                            },
                            Err(IncrementRateError::RateLimitOver) => Err(RateLimitError::RateLimitOver),
                            _ => Err(RateLimitError::RateLimitFailed),
                        }
                    },
                    None => Err(RateLimitError::InvalidApiKey),
                }
            },
            None => Err(RateLimitError::NoApiKey)
        }
    }

    fn extract_no_account_user_api_key(headers: &HeaderMap) -> Option<ApiKey> {
        headers.get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .and_then(|token| ApiKey::from_str(token).ok())
    }

    async fn fetch_last_api_key_refreshed_at(&self, api_key: &ApiKey) -> Fallible<Option<LastApiKeyRefreshedAt>, RateLimitError>;
}

#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("APIキーがありません")]
    NoApiKey,
    #[error("無効なAPIキーです")]
    InvalidApiKey,
    #[error("APIキーの存在確認に失敗しました")]
    FetchLastApiKeyRefreshedAt(#[source] anyhow::Error),
    #[error("レート上限に達しています")]
    RateLimitOver,
    #[error("レート制限に失敗しました")]
    RateLimitFailed,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LastApiKeyRefreshedAt(UnixtimeMillis);

impl LastApiKeyRefreshedAt {
    pub fn new(unixtime: UnixtimeMillis) -> Self {
        Self(unixtime)
    }

    pub fn value(&self) -> &UnixtimeMillis {
        &self.0
    }
}

impl FromCqlVal<Option<CqlValue>> for LastApiKeyRefreshedAt {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        UnixtimeMillis::from_cql(cql_val).map(Self::new)
    }
}

#[cfg(test)]
mod tests {
    use std::{convert::Infallible, future::{ready, Ready}, sync::LazyLock, task::{Context, Poll}};

    use http::{Request, Response};
    use thiserror::Error;
    use tower::Service;

    use crate::{common::{api_key::ApiKey, fallible::Fallible, unixtime::UnixtimeMillis}, middlewares::rate_limit::dsl::{increment_rate::{IncrementRate, IncrementRateError, InculsiveLimit, Rate, TimeWindow}, refresh_api_key::{ApiKeyExpirationSeconds, ApiKeyRefreshThereshold, RefreshApiKey, RefreshApiKeyError}}};

    use super::{LastApiKeyRefreshedAt, RateLimit, RateLimitError};

    static VALID_API_KEY: LazyLock<ApiKey> = LazyLock::new(|| ApiKey::gen());

    struct MockRateLimit;

    impl RateLimit for MockRateLimit {
        async fn fetch_last_api_key_refreshed_at(&self, api_key: &ApiKey) -> Fallible<Option<LastApiKeyRefreshedAt>, RateLimitError> {
            if api_key == &*VALID_API_KEY {
                Ok(Some(LastApiKeyRefreshedAt::new(UnixtimeMillis::now())))
            } else {
                Ok(None)
            }
        }
    }

    const TIME_WINDOW: TimeWindow = TimeWindow::secs(60);
    const INCLUSIVE_LIMIT: InculsiveLimit = InculsiveLimit::new(100);

    impl IncrementRate for MockRateLimit {
        async fn increment_rate_within_window(&self, api_key: &ApiKey, _: &TimeWindow) -> Fallible<Rate, IncrementRateError> {
            if api_key == &*VALID_API_KEY {
                Ok(Rate::new(0))
            } else {
                Err(IncrementRateError::RateLimitOver)
            }
        }

        fn time_window(&self) -> &TimeWindow {
            &TIME_WINDOW
        }
    
        fn inclusive_limit(&self) -> &InculsiveLimit {
            &INCLUSIVE_LIMIT
        }
    }

    #[derive(Debug, Error)]
    #[error("疑似エラー")]
    struct MockError;

    const API_KEY_REFRESH_THERESHOLD: ApiKeyRefreshThereshold = ApiKeyRefreshThereshold::days(10);
    const API_KEY_EXPIRATION: ApiKeyExpirationSeconds = ApiKeyExpirationSeconds::secs(60);

    impl RefreshApiKey for MockRateLimit {
        fn api_key_refresh_thereshold(&self) -> ApiKeyRefreshThereshold {
            API_KEY_REFRESH_THERESHOLD
        }

        fn api_key_expiration(&self) -> ApiKeyExpirationSeconds {
            API_KEY_EXPIRATION
        }

        async fn refresh_api_key(&self, api_key: &ApiKey, _expiration: ApiKeyExpirationSeconds) -> Fallible<(), RefreshApiKeyError> {
            if api_key == &*VALID_API_KEY {
                Ok(())
            } else {
                Err(RefreshApiKeyError::RefreshApiKeyFailed(MockError.into()))
            }
        }
    }

    struct MockService;

    impl Service<Request<()>> for MockService {
        type Response = Response<()>;
        type Error = Infallible;
        type Future = Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _: Request<()>) -> Self::Future {
            ready(Ok(Response::new(())))
        }
    }

    async fn test_rate_limit(api_key: &ApiKey) -> Fallible<Response<()>, RateLimitError> {
        let request = Request::builder()
            .header("Authorization", format!("Bearer {}", api_key.to_string()))
            .body(())
            .unwrap();
        MockRateLimit.rate_limit(&mut MockService, request).await
    }

    #[tokio::test]
    async fn valid_api_key() {
        let result = test_rate_limit(&*VALID_API_KEY).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn invalid_api_key() {
        let result = test_rate_limit(&ApiKey::gen()).await;
        assert!(matches!(result, Err(RateLimitError::InvalidApiKey)));
    }
}