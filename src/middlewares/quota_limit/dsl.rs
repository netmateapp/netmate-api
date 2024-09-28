use std::convert::Infallible;

use http::{Request, Response, StatusCode};
use thiserror::Error;
use tower::Service;

use crate::{common::{fallible::Fallible, id::account_id::AccountId}, middlewares::limit::{Count, InculsiveLimit, TimeWindow}};

pub type ConsumedQuota = Count;

pub(crate) trait QuotaLimit {
    async fn quota_limit<S, B>(&self, inner: &mut S, request: Request<B>) -> Fallible<S::Response, QuotaLimitError>
    where
        S: Service<Request<B>, Error = Infallible, Response = Response<B>>
    {
        let account_id = request.extensions()
            .get::<AccountId>()
            .cloned()
            .ok_or_else(|| QuotaLimitError::QuotaLimitFailed)?;

        if self.is_limit_over(account_id).await? {
            Err(QuotaLimitError::QuotaLimitOver)
        } else {
            // `Error`は`Infallible`であるため`unwrap()`で問題ない
            let response = inner.call(request).await.unwrap();

            match response.status() {
                StatusCode::OK => {
                    // 失敗しても続行
                    let _ = self.increment_consumed_quota(account_id, self.time_window()).await;
                    
                    Ok(response)
                },
                _ => Err(QuotaLimitError::QuotaLimitFailed)
            }
        }
    }

    async fn is_limit_over(&self, account_id: AccountId) -> Fallible<bool, QuotaLimitError> {
        match self.fetch_personal_limit(account_id).await? {
            Some(personal_limit) => {
                let consumed_quota = self.fetch_consumed_quota(account_id)
                    .await?
                    .unwrap_or_else(|| ConsumedQuota::new(0));
    
                Ok(personal_limit.value() <= consumed_quota)
            },
            None => Ok(true)
        }
    }

    async fn fetch_personal_limit(&self, account_id: AccountId) -> Fallible<Option<InculsiveLimit>, QuotaLimitError>;

    async fn fetch_consumed_quota(&self, account_id: AccountId) -> Fallible<Option<ConsumedQuota>, QuotaLimitError>;

    async fn increment_consumed_quota(&self, account_id: AccountId, time_window: TimeWindow) -> Fallible<(), QuotaLimitError>;

    fn time_window(&self) -> TimeWindow;
}

#[derive(Debug, Error)]
pub enum QuotaLimitError {
    #[error("個人のクォータ上限の取得に失敗しました")]
    FetchPersonalLimitFailed(#[source] anyhow::Error),
    #[error("消費クォータの取得に失敗しました")]
    FetchConsumedQuotaFailed(#[source] anyhow::Error),
    #[error("クォータ上限に達しています")]
    QuotaLimitOver,
    #[error("消費クォータのインクリメントに失敗しました")]
    IncrementConsumedQuotaFailed(#[source] anyhow::Error),
    #[error("クォータ制限に失敗しました")]
    QuotaLimitFailed,
}

#[cfg(test)]
mod tests {
    use std::{convert::Infallible, future::{ready, Ready}, sync::LazyLock, task::{Context, Poll}};

    use http::{Request, Response};
    use thiserror::Error;
    use tower::Service;


    use crate::{common::{fallible::Fallible, id::account_id::AccountId}, middlewares::limit::{InculsiveLimit, TimeWindow}};

    use super::{ConsumedQuota, QuotaLimit, QuotaLimitError};

    struct MockQuotaLimit;

    #[derive(Debug, Error)]
    #[error("疑似エラー")]
    struct MockError;

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

    static UNCONSUMED: LazyLock<AccountId> = LazyLock::new(AccountId::gen);
    static WITHIN_LIMIT: LazyLock<AccountId> = LazyLock::new(AccountId::gen);
    static LIMIT_OVER: LazyLock<AccountId> = LazyLock::new(AccountId::gen);

    impl QuotaLimit for MockQuotaLimit {
        async fn fetch_personal_limit(&self, account_id: AccountId) -> Fallible<Option<InculsiveLimit>, QuotaLimitError> {
            if account_id == *UNCONSUMED || account_id == *WITHIN_LIMIT || account_id == *LIMIT_OVER {
                Ok(Some(InculsiveLimit::new(ConsumedQuota::new(2))))
            } else {
                Ok(None)
            }
        }

        async fn fetch_consumed_quota(&self, account_id: AccountId) -> Fallible<Option<ConsumedQuota>, QuotaLimitError> {
            if account_id == *UNCONSUMED {
                Ok(None)
            } else if account_id == *WITHIN_LIMIT {
                Ok(Some(ConsumedQuota::new(1)))
            } else if account_id == *LIMIT_OVER {
                Ok(Some(ConsumedQuota::new(2)))
            } else {
                Err(QuotaLimitError::FetchConsumedQuotaFailed(MockError.into()))
            }
        }
    
        async fn increment_consumed_quota(&self, _: AccountId, _: TimeWindow) -> Fallible<(), QuotaLimitError> {
            Ok(())
        }
    
        fn time_window(&self) -> TimeWindow {
            TimeWindow::seconds(60)
        }
    }

    async fn test_quota_limit(account_id: AccountId) -> Fallible<Response<()>, QuotaLimitError> {
        let mut request = Request::builder()
            .body(())
            .unwrap();

        // セッション管理ミドルウェアを模倣
        request.extensions_mut().insert(account_id);

        MockQuotaLimit.quota_limit(&mut MockService, request).await
    }

    #[tokio::test]
    async fn no_personal_limit() {
        let res = test_quota_limit(AccountId::gen()).await;
        assert!(matches!(res.err().unwrap(), QuotaLimitError::QuotaLimitOver));
    }

    #[tokio::test]
    async fn unconsumed() {
        let res = test_quota_limit(*UNCONSUMED).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn within_limit() {
        let res = test_quota_limit(*WITHIN_LIMIT).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn limit_over() {
        let res = test_quota_limit(*LIMIT_OVER).await;
        assert!(matches!(res.err().unwrap(), QuotaLimitError::QuotaLimitOver));
    }
}