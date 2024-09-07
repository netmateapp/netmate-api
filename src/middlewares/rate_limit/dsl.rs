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
                            None => Err(LimitRateError::NoApiKey)
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

#[derive(Debug, Error)]
pub enum LimitRateError {
    #[error("APIキーがありません")]
    NoApiKey,
    #[error("レート上限に達しています")]
    RateLimitOver,
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
