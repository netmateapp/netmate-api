use std::{convert::Infallible, future::Future, pin::Pin, sync::Arc, task::{ready, Context, Poll}};

use http::{Request, Response, StatusCode};
use pin_project::pin_project;
use scylla::Session;
use tokio::pin;
use tower::{Layer, Service};

use crate::{helper::{error::InitError, redis::Pool}, middlewares::start_session::dsl::start_session::StartSession};

use super::interpreter::StartSessionImpl;

#[derive(Clone)]
pub struct StartSessionLayer {
    start_session: Arc<StartSessionImpl>,
}

impl StartSessionLayer {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<StartSessionImpl>> {
        let start_session = StartSessionImpl::try_new(db, cache).await?;
        Ok(Self { start_session: Arc::new(start_session) })
    }
}

impl<S> Layer<S> for StartSessionLayer {
    type Service = StartSessionService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        StartSessionService {
            inner,
            start_session: self.start_session.clone(),
        }
    }
}

#[derive(Clone)]
pub struct StartSessionService<S> {
    inner: S,
    start_session: Arc<StartSessionImpl>,
}

impl <S, B> Service<Request<B>> for StartSessionService<S>
where
    S: Service<Request<B>, Error = Infallible, Response = Response<B>> + Clone,
    S::Future: Future<Output = Result<S::Response, S::Error>>,
    B: Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = StartSessionFuture<S, B>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        StartSessionFuture {
            inner: self.inner.clone(), // 下記の都合で`Future::poll`内で`inner.call(req)`を呼ぶ必要があるため複製して渡す
            request: Some(req), // `inner.call(req)`が`req`の所有権を必要とするため渡す必要がある
            start_session: self.start_session.clone(),
        }
    }
}

#[pin_project]
pub struct StartSessionFuture<S, B>
where
    S: Service<Request<B>>,
    B: Default,
{
    inner: S,
    request: Option<Request<B>>,
    start_session: Arc<StartSessionImpl>,
}

impl<S, B> Future for StartSessionFuture<S, B>
where
    S: Service<Request<B>, Error = Infallible, Response = Response<B>>,
    S::Future: Future<Output = Result<Response<B>, S::Error>>,
    B: Default,
{
    type Output = Result<S::Response, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let response_future = this.start_session
            .start_session::<S, B>(this.inner, this.request.take().unwrap());
        pin!(response_future);

        // エラーもレスポンスに変換して返す
        match ready!(response_future.poll(cx)) {
            Ok(response) => Poll::Ready(Ok(response)),
            Err(_) => {
                let response = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(B::default())
                    .unwrap();
                Poll::Ready(Ok(response))
            }
        }
    }
}