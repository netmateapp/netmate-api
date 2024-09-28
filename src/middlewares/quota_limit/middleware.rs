use std::{convert::Infallible, future::Future, pin::Pin, sync::Arc, task::{ready, Context, Poll}};

use http::{Request, Response, StatusCode};
use pin_project::pin_project;
use scylla::Session;
use tokio::pin;
use tower::{Layer, Service};

use crate::{helper::{error::InitError, redis::Pool}, middlewares::{limit::{EndpointName, TimeWindow}, quota_limit::dsl::{QuotaLimit, QuotaLimitError}}};

use super::interpreter::QuotaLimitImpl;

#[derive(Clone)]
pub struct QuotaLimitLayer {
    quota_limit: Arc<QuotaLimitImpl>,
}

impl QuotaLimitLayer {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>, endpoint_name: EndpointName, time_window: TimeWindow) -> Result<Self, InitError<QuotaLimitImpl>> {
        let quota_limit = QuotaLimitImpl::try_new(db, cache, endpoint_name, time_window).await?;
        Ok(Self { quota_limit: Arc::new(quota_limit) })
    }
}

impl<S> Layer<S> for QuotaLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            quota_limit: self.quota_limit.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    quota_limit: Arc<QuotaLimitImpl>,
}

impl <S, B> Service<Request<B>> for RateLimitService<S>
where
    S: Service<Request<B>, Error = Infallible, Response = Response<B>> + Clone,
    S::Future: Future<Output = Result<S::Response, S::Error>>,
    B: Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = SessionFuture<S, B>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        SessionFuture {
            inner: self.inner.clone(), // 下記の都合で`Future::poll`内で`inner.call(req)`を呼ぶ必要があるため複製して渡す
            request: Some(req), // `inner.call(req)`が`req`の所有権を必要とするため渡す必要がある
            quota_limit: self.quota_limit.clone(),
        }
    }
}

#[pin_project]
pub struct SessionFuture<S, B>
where
    S: Service<Request<B>>,
    B: Default,
{
    inner: S,
    request: Option<Request<B>>,
    quota_limit: Arc<QuotaLimitImpl>,
}

impl<S, B> Future for SessionFuture<S, B>
where
    S: Service<Request<B>, Error = Infallible, Response = Response<B>>,
    S::Future: Future<Output = Result<S::Response, S::Error>>,
    B: Default,
{
    type Output = Result<S::Response, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let response_future = this.quota_limit
            .quota_limit::<S, B>(this.inner, this.request.take().unwrap());
        pin!(response_future);

        // エラーもレスポンスに変換して返す
        match ready!(response_future.poll(cx)) {
            Ok(response) => Poll::Ready(Ok(response)),
            Err(e) => {
                let status_code = match e {
                    QuotaLimitError::QuotaLimitOver => StatusCode::TOO_MANY_REQUESTS,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };

                let response = Response::builder()
                    .status(status_code)
                    .body(B::default())
                    .unwrap();

                Poll::Ready(Ok(response))
            }
        }
    }
}