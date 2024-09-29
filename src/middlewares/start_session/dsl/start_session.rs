use std::convert::Infallible;

use http::{Request, Response};
use thiserror::Error;
use tower::Service;

use crate::common::{fallible::Fallible, profile::account_id::AccountId, session::{cookie::{set_refresh_pair_cookie_with_expiration, set_session_cookie_with_expiration}, refresh_pair_expiration::REFRESH_PAIR_EXPIRATION, session_expiration::SESSION_EXPIRATION}};

use super::{assign_refresh_pair::{AssignRefreshPair, AssignRefreshPairError}, assign_session_id::{AssignSessionId, AssignSessionIdError}};

pub(crate) trait StartSession {
    async fn start_session<S, B>(&self, inner: &mut S, request: Request<B>) -> Fallible<S::Response, StartSessionError>
    where
        Self: AssignSessionId + AssignRefreshPair,
        S: Service<Request<B>, Error = Infallible, Response = Response<B>>,
    {
        // `Infallible`であるため`unwrap`しても問題ない
        let mut response = inner.call(request)
            .await
            .unwrap();

        let session_account_id = response.extensions()
            .get::<AccountId>()
            .cloned();

        match session_account_id {
            Some(session_account_id) => {
                let session_id = self.assign_session_id(session_account_id, SESSION_EXPIRATION).await?;
                set_session_cookie_with_expiration(&mut response, &session_id);
        
                let (session_series, refresh_token) = self.assign_refresh_pair(session_account_id, REFRESH_PAIR_EXPIRATION).await?;
                set_refresh_pair_cookie_with_expiration(&mut response, &session_series, &refresh_token);
                
                Ok(response)
            },
            None => Ok(response)
        }
    }
}

#[derive(Debug, Error)]
pub enum StartSessionError {
    #[error("セッションIDの割り当てに失敗しました")]
    AssignSessionIdFailed(#[from] AssignSessionIdError),
    #[error("リフレッシュペアの割り当てに失敗しました")]
    AssignRefreshPairFailed(#[from] AssignRefreshPairError),
}

#[cfg(test)]
mod tests {
    use std::{convert::Infallible, future::{ready, Ready}, task::{Context, Poll}};

    use http::{header::SET_COOKIE, Request, Response, StatusCode};
    use tower::Service;

    use crate::{common::{fallible::Fallible, profile::account_id::AccountId, session::{refresh_pair_expiration::RefreshPairExpirationSeconds, refresh_token::RefreshToken, session_expiration::SessionExpirationSeconds, session_id::SessionId, session_series::SessionSeries}}, middlewares::start_session::dsl::{assign_refresh_pair::{AssignRefreshPair, AssignRefreshPairError}, assign_session_id::{AssignSessionId, AssignSessionIdError}}};

    use super::StartSession;

    struct MockStartSession;

    impl StartSession for MockStartSession {}

    impl AssignSessionId for MockStartSession {
        async fn try_assign_new_session_id_with_expiration_if_unused(&self, _: &SessionId, _: AccountId, _: SessionExpirationSeconds) -> Fallible<(), AssignSessionIdError> {
            Ok(())
        }
    }

    impl AssignRefreshPair for MockStartSession {
        async fn try_assign_refresh_pair_with_expiration_if_unused(&self, _: &SessionSeries, _: &RefreshToken, _: AccountId, _: RefreshPairExpirationSeconds) -> Fallible<(), AssignRefreshPairError> {
            Ok(())
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

        // 下位ハンドラを模倣し、成功時のみアカウントIDをextensionにセットする
        fn call(&mut self, request: Request<()>) -> Self::Future {
            if let Some(account_id) = request.extensions().get::<AccountId>() {
                let mut response = Response::new(());
                response.extensions_mut().insert(*account_id);
                ready(Ok(response))
            } else {
                let response = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(())
                    .unwrap();
                ready(Ok(response))
            }
        }
    }

    async fn test_start_session(account_id: Option<AccountId>, expected_set_cookie_headers: usize) {
        let mut request = Request::new(());
        if let Some(account_id) = account_id {
            request.extensions_mut().insert(account_id);
        }

        let response = MockStartSession.start_session(&mut MockService, request).await.unwrap();
        let set_cookie_headers = response.headers().get_all(SET_COOKIE).iter().count();

        assert_eq!(set_cookie_headers, expected_set_cookie_headers);
    }

    #[tokio::test]
    async fn start_session_success() {
        test_start_session(Some(AccountId::gen()), 2).await;
    }

    #[tokio::test]
    async fn start_session_failure() {
        test_start_session(None, 0).await;
    }
}