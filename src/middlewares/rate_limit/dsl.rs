use std::{convert::Infallible, str::FromStr};

use http::{HeaderMap, Request, Response};
use thiserror::Error;
use tower::Service;

use crate::common::{api_key::ApiKey, fallible::Fallible, unixtime::UnixtimeMillis};

pub(crate) trait RateLimit {
    async fn rate_limit<S, B>(&self, inner: &mut S, request: Request<B>) -> Fallible<S::Response, RateLimitError>
    where
        S: Service<Request<B>, Error = Infallible, Response = Response<B>>
    {
        match extract_no_account_user_api_key(request.headers()) {
            Some(api_key) => {
                match self.check_api_key_exists(&api_key).await? {
                    Some(last_api_key_refreshed_at) => {
                        let rate = self.increment_rate(&api_key).await?;
                        if self.is_limit_over(rate) {
                            Err(RateLimitError::RateLimitOver)
                        } else {
                            // `Error`は`Infallible`であるため`unwrap()`で問題ない
                            let response = inner.call(request).await.unwrap();

                            if self.should_refresh_api_key(&last_api_key_refreshed_at) {
                                // エラーが発生しても続行
                                let _ = self.refresh_api_key(&api_key);
                            }

                            Ok(response)
                        }
                    },
                    None => Err(RateLimitError::InvalidApiKey),
                }
            },
            None => Err(RateLimitError::NoApiKey)
        }
    }

    async fn check_api_key_exists(&self, api_key: &ApiKey) -> Fallible<Option<ApiKeyRefreshTimestamp>, RateLimitError>;

    async fn increment_rate(&self, api_key: &ApiKey) -> Fallible<u16, RateLimitError>;

    fn is_limit_over(&self, rate: u16) -> bool {
        rate > self.limit()
    }

    fn limit(&self) -> u16;

    fn should_refresh_api_key(&self, api_key_refresh_timestamp: &ApiKeyRefreshTimestamp) -> bool;

    async fn refresh_api_key(&self, api_key: &ApiKey) -> Fallible<(), RateLimitError>;

}

#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("APIキーがありません")]
    NoApiKey,
    #[error("レートの取得に失敗しました")]
    IncrementRateFailed(#[source] anyhow::Error),
    #[error("レート上限に達しています")]
    RateLimitOver,
    #[error("APIキーの存在確認に失敗しました")]
    CheckApiKeyExistsFailed(#[source] anyhow::Error),
    #[error("APIキーのTTLのリフレッシュに失敗しました")]
    RefreshApiKeyFailed(#[source] anyhow::Error),
    #[error("無効なAPIキーです")]
    InvalidApiKey,
}

fn extract_no_account_user_api_key(headers: &HeaderMap) -> Option<ApiKey> {
    headers.get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .and_then(|token| ApiKey::from_str(token).ok())
}

pub struct ApiKeyRefreshTimestamp(UnixtimeMillis);

impl ApiKeyRefreshTimestamp {
    pub fn new(unixtime: UnixtimeMillis) -> Self {
        Self(unixtime)
    }

    pub fn value(&self) -> &UnixtimeMillis {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::{convert::Infallible, future::{ready, Ready}, sync::LazyLock, task::{Context, Poll}};

    use http::{HeaderValue, Request, Response};
    use tower::Service;

    use crate::common::{api_key::ApiKey, fallible::Fallible, unixtime::UnixtimeMillis};

    use super::{ApiKeyRefreshTimestamp, RateLimitError, RateLimit};

    struct MockRateLimit;

    static WITHIN_LIMIT_API_KEY: LazyLock<ApiKey> = LazyLock::new(|| ApiKey::gen());
    static OVER_LIMIT_API_KEY: LazyLock<ApiKey> = LazyLock::new(|| ApiKey::gen());
    static INVALID_API_KEY: LazyLock<ApiKey> = LazyLock::new(|| ApiKey::gen());

    const LIMIT: u16 = 5;

    impl RateLimit for MockRateLimit {
        async fn check_api_key_exists(&self, api_key: &ApiKey) -> Fallible<Option<ApiKeyRefreshTimestamp>, RateLimitError> {
            if api_key == &*WITHIN_LIMIT_API_KEY || api_key == &*OVER_LIMIT_API_KEY {
                Ok(Some(ApiKeyRefreshTimestamp::new(UnixtimeMillis::now())))
            } else {
                Ok(None)
            }
        }

        async fn increment_rate(&self, api_key: &ApiKey) -> Fallible<u16, RateLimitError> {
            if api_key == &*WITHIN_LIMIT_API_KEY {
                Ok(LIMIT - 1)
            } else if api_key == &*OVER_LIMIT_API_KEY {
                Ok(LIMIT + 1)
            } else {
                Ok(1)
            }
        }

        fn limit(&self) -> u16 {
            LIMIT
        }

        fn should_refresh_api_key(&self, _api_key_refresh_timestamp: &ApiKeyRefreshTimestamp) -> bool {
            true
        }

        async fn refresh_api_key(&self, _api_key: &ApiKey) -> Fallible<(), RateLimitError> {
            Ok(())
        }
    }

    struct MockInnerService;

    impl Service<Request<()>> for MockInnerService {
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

    async fn test_rate_limit(api_key: Option<&ApiKey>) -> Fallible<Response<()>, RateLimitError> {
        let mut inner = MockInnerService;

        let mut request = Request::new(());
        if let Some(api_key) = api_key {
            let header_value: HeaderValue = format!("Bearer {}", api_key.value().value())
                .parse()
                .unwrap();
            request.headers_mut().insert("Authorization", header_value);
        }

        MockRateLimit.rate_limit(&mut inner, request).await
    }

    #[tokio::test]
    async fn test_rate_limit_within_limit() {
        let response = test_rate_limit(Some(&*WITHIN_LIMIT_API_KEY)).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limit_over_limit() {
        let response = test_rate_limit(Some(&*OVER_LIMIT_API_KEY)).await;
        match response.unwrap_err() {
            RateLimitError::RateLimitOver => (),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_rate_limit_invalid_api_key() {
        let response = test_rate_limit(Some(&*INVALID_API_KEY)).await;
        match response.unwrap_err() {
            RateLimitError::InvalidApiKey => (),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_rate_limit_no_api_key() {
        let response = test_rate_limit(None).await;
        match response.unwrap_err() {
            RateLimitError::NoApiKey => (),
            e => panic!("Unexpected error: {:?}", e),
        }
    }
}
