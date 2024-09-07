use std::{convert::Infallible, str::FromStr};

use http::{HeaderMap, Request, Response};
use thiserror::Error;
use tower::Service;

use crate::common::{api_key::ApiKey, fallible::Fallible, unixtime::UnixtimeMillis};

pub(crate) trait RateLimit {
    async fn rate_limit<S, B>(&self, inner: &mut S, request: Request<B>) -> Fallible<S::Response, LimitRateError>
    where
        S: Service<Request<B>, Error = Infallible, Response = Response<B>>
    {
        match extract_no_account_user_api_key(request.headers()) {
            Some(api_key) => {
                match self.get_rate(&api_key).await? {
                    Some(rate) => {
                        if self.is_limit_over(rate) {
                            Err(LimitRateError::RateLimitOver)
                        } else {
                            // `Error`は`Infallible`であるため`unwrap()`で問題ない
                            let response = inner.call(request).await.unwrap();
                            Ok(response)
                        }
                    },
                    None => {
                        match self.check_api_key_exists(&api_key).await? {
                            Some(api_key_refresh_timestamp) => {
                                if self.should_refresh_api_key(&api_key_refresh_timestamp) {
                                    // エラーが発生しても続行
                                    let _ = self.refresh_api_key(&api_key).await;
                                }
                                // `Error`は`Infallible`であるため`unwrap()`で問題ない
                                let response = inner.call(request).await.unwrap();
                                Ok(response)
                            },
                            None => Err(LimitRateError::InvalidApiKey)
                        }
                    }
                }
            },
            None => Err(LimitRateError::NoApiKey)
        }
    }

    async fn get_rate(&self, api_key: &ApiKey) -> Fallible<Option<u16>, LimitRateError>;

    fn is_limit_over(&self, rate: u16) -> bool;

    // 複数のエンドポイントで同じ検証をするのは効率が悪いので、
    // 30分～1時間程度の短時間キャッシュを行うべき(リフレッシュ時刻も併せてキャッシュするため、短時間にする必要がある)
    async fn check_api_key_exists(&self, api_key: &ApiKey) -> Fallible<Option<ApiKeyRefreshTimestamp>, LimitRateError>;

    fn should_refresh_api_key(&self, api_key_refresh_timestamp: &ApiKeyRefreshTimestamp) -> bool;

    async fn refresh_api_key(&self, api_key: &ApiKey) -> Fallible<(), LimitRateError>;

}

#[derive(Debug, Error, PartialEq)]
pub enum LimitRateError {
    #[error("APIキーがありません")]
    NoApiKey,
    #[error("レート上限に達しています")]
    RateLimitOver,
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

    use super::{ApiKeyRefreshTimestamp, LimitRateError, RateLimit};

    struct MockRateLimit;

    static WITHIN_LIMIT_API_KEY: LazyLock<ApiKey> = LazyLock::new(|| ApiKey::gen());
    static OVER_LIMIT_API_KEY: LazyLock<ApiKey> = LazyLock::new(|| ApiKey::gen());
    static NO_RATE_API_KEY: LazyLock<ApiKey> = LazyLock::new(|| ApiKey::gen());
    static INVALID_API_KEY: LazyLock<ApiKey> = LazyLock::new(|| ApiKey::gen());

    const LIMIT: u16 = 5;

    impl RateLimit for MockRateLimit {
        async fn get_rate(&self, api_key: &ApiKey) -> Fallible<Option<u16>, LimitRateError> {
            if api_key == &*WITHIN_LIMIT_API_KEY {
                Ok(Some(LIMIT - 1))
            } else if api_key == &*OVER_LIMIT_API_KEY {
                Ok(Some(LIMIT + 1))
            } else {
                Ok(None)
            }
        }

        fn is_limit_over(&self, rate: u16) -> bool {
            rate > LIMIT
        }

        async fn check_api_key_exists(&self, api_key: &ApiKey) -> Fallible<Option<ApiKeyRefreshTimestamp>, LimitRateError> {
            if api_key == &*NO_RATE_API_KEY {
                Ok(Some(ApiKeyRefreshTimestamp::new(UnixtimeMillis::now())))
            } else {
                Ok(None)
            }
        }

        fn should_refresh_api_key(&self, _api_key_refresh_timestamp: &ApiKeyRefreshTimestamp) -> bool {
            true
        }

        async fn refresh_api_key(&self, _api_key: &ApiKey) -> Fallible<(), LimitRateError> {
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

    async fn test_rate_limit(api_key: Option<&ApiKey>) -> Fallible<Response<()>, LimitRateError> {
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
        assert_eq!(response.unwrap_err(), LimitRateError::RateLimitOver);
    }

    #[tokio::test]
    async fn test_rate_limit_no_rate() {
        let response = test_rate_limit(Some(&*NO_RATE_API_KEY)).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limit_invalid_api_key() {
        let response = test_rate_limit(Some(&*INVALID_API_KEY)).await;
        assert_eq!(response.unwrap_err(), LimitRateError::InvalidApiKey);
    }

    #[tokio::test]
    async fn test_rate_limit_no_api_key() {
        let response = test_rate_limit(None).await;
        assert_eq!(response.unwrap_err(), LimitRateError::NoApiKey);
    }
}
