use std::{convert::Infallible, future::Future, pin::Pin, sync::Arc, task::{ready, Context, Poll}};

use http::{Request, Response};
use pin_project::pin_project;
use scylla::Session;
use tokio::pin;
use tower::{Layer, Service};

use crate::helper::{error::InitError, garnet::Pool};

use super::{dsl::{ManageSession, ManageSessionError}, interpreter::ManageSessionImpl};

#[derive(Clone)]
pub struct LoginSessionLayer {
    manage_session: Arc<ManageSessionImpl>,
}

impl LoginSessionLayer {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<ManageSessionImpl>> {
        let manage_session = ManageSessionImpl::try_new(db, cache).await?;
        Ok(Self { manage_session: Arc::new(manage_session) })
    }
}

impl<S> Layer<S> for LoginSessionLayer {
    type Service = LoginSessionService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoginSessionService {
            inner,
            manage_session: self.manage_session.clone(),
        }
    }
}

#[derive(Clone)]
pub struct LoginSessionService<S> {
    inner: S,
    manage_session: Arc<ManageSessionImpl>,
}

impl <S, B> Service<Request<B>> for LoginSessionService<S>
where
    S: Service<Request<B>, Error = Infallible, Response = Response<B>> + Clone,
    S::Future: Future<Output = Result<S::Response, S::Error>>,
{
    type Response = S::Response;
    type Error = ManageSessionError;
    type Future = SessionFuture<S, B>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|_| ManageSessionError::NoSession)
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
{
    inner: S,
    request: Option<Request<B>>,
    manage_session: Arc<ManageSessionImpl>,
}

impl<S, B> Future for SessionFuture<S, B>
where
    S: Service<Request<B>, Error = Infallible, Response = Response<B>>,
    S::Future: Future<Output = Result<Response<B>, S::Error>>,
{
    type Output = Result<Response<B>, ManageSessionError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let response_future = this.manage_session
            .manage_session::<S, B>(this.inner, this.request.take().unwrap());
        pin!(response_future);
        match ready!(response_future.poll(cx)) {
            Ok(response) => Poll::Ready(Ok(response)),
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}