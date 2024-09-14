use std::{convert::Infallible, future::Future, pin::Pin, sync::Arc, task::{ready, Context, Poll}};

use http::{Request, Response, StatusCode};
use pin_project::pin_project;
use scylla::Session;
use tokio::pin;
use tower::{Layer, Service};

use crate::{helper::{error::InitError, redis::Pool}, middlewares::manage_session::dsl::manage_session::{ManageSession, ManageSessionError}};

use super::interpreter::ManageSessionImpl;

#[derive(Clone)]
pub struct ManageSessionLayer {
    manage_session: Arc<ManageSessionImpl>,
}

impl ManageSessionLayer {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<ManageSessionImpl>> {
        let manage_session = ManageSessionImpl::try_new(db, cache).await?;
        Ok(Self { manage_session: Arc::new(manage_session) })
    }
}

impl<S> Layer<S> for ManageSessionLayer {
    type Service = ManageSessionService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ManageSessionService {
            inner,
            manage_session: self.manage_session.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ManageSessionService<S> {
    inner: S,
    manage_session: Arc<ManageSessionImpl>,
}

impl <S, B> Service<Request<B>> for ManageSessionService<S>
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
            manage_session: self.manage_session.clone(),
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
    manage_session: Arc<ManageSessionImpl>,
}

impl<S, B> Future for SessionFuture<S, B>
where
    S: Service<Request<B>, Error = Infallible, Response = Response<B>>,
    S::Future: Future<Output = Result<Response<B>, S::Error>>,
    B: Default,
{
    type Output = Result<S::Response, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let response_future = this.manage_session
            .manage_session::<S, B>(this.inner, this.request.take().unwrap());
        pin!(response_future);

        // エラーもレスポンスに変換して返す
        match ready!(response_future.poll(cx)) {
            Ok(response) => Poll::Ready(Ok(response)),
            Err(e) => {
                let status_code = match e {
                    ManageSessionError::NoSession => StatusCode::UNAUTHORIZED,
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