use std::convert::Infallible;

use http::{header::SET_COOKIE, Request, Response};
use thiserror::Error;
use tower::Service;

use crate::common::{fallible::Fallible, session::{cookie::{set_refresh_pair_cookie_with_expiration, set_session_cookie_with_expiration}, refresh_pair_expiration::REFRESH_PAIR_EXPIRATION, session_expiration::SESSION_EXPIRATION}};

use super::{authenticate::AuthenticateSession, extract_session_info::ExtractSessionInformation, mitigate_session_theft::MitigateSessionTheft, reauthenticate::{ReAuthenticateSession, ReAuthenticateSessionError}, refresh_session_series::RefreshSessionSeries, update_refresh_token::UpdateRefreshToken, update_session::UpdateSession};

pub(crate) trait ManageSession {
    async fn manage_session<S, B>(&self, inner: &mut S, mut request: Request<B>) -> Fallible<S::Response, ManageSessionError>
    where
        Self: ExtractSessionInformation + AuthenticateSession + ReAuthenticateSession + UpdateSession + UpdateRefreshToken + RefreshSessionSeries + MitigateSessionTheft,
        S: Service<Request<B>, Error = Infallible, Response = Response<B>>,
    {
        let (session_id, pair) = Self::extract_session_information(&request);

        if session_id.is_none() && pair.is_none() {
            return Err(ManageSessionError::NoSession);
        }

        if let Some(session_id) = session_id {
            if let Ok(account_id) = self.authenticate_session(&session_id).await {
                request.extensions_mut().insert(account_id);

                // `Error`は`Infallible`で起こり得ないので`unwrap()`で問題ない
                let mut response = inner.call(request).await.unwrap();

                // パスワード変更やログアウトによるSet-Cookieヘッダが無い場合のみセッションを延長
                if !response.headers().contains_key(SET_COOKIE) {
                    // 同じセッションIDをセットすることで有効期限をリフレッシュ
                    set_session_cookie_with_expiration(&mut response, &session_id);
                }
                
                return Ok(response)
            }
        }
    
        if let Some((session_series, refresh_token)) = pair {
            match self.reauthenticate_session(&session_series, refresh_token).await {
                Ok(account_id) => {
                    request.extensions_mut().insert(account_id);

                    // `Error`は`Infallible`で起こり得ないので`unwrap()`で問題ない
                    let mut response = inner.call(request).await.unwrap();
            
                    // パスワード変更やログアウトによるSet-Cookieヘッダが無い場合のみセッションを延長
                    if !response.headers().contains_key(SET_COOKIE) {
                        // セッションIDの更新に成功した場合のみに限定することで、
                        // 基本的に最低30分は間隔を空けて更新処理を行うようにし負荷を抑える
                        // ※セッションIDを破棄して送信されるリクエストへの耐性は無い
                        if let Ok(new_session_id) = self.update_session(account_id, SESSION_EXPIRATION).await {
                            set_session_cookie_with_expiration(&mut response, &new_session_id);

                            // リフレッシュトークンの発行が失敗した場合は、現在のトークンを使用し続ける
                            // これはセキュリティリスクを多少増加させるが許容の範囲内である
                            if let Ok(new_refresh_token) = self.update_refresh_token(&session_series, account_id, REFRESH_PAIR_EXPIRATION).await { 
                                set_refresh_pair_cookie_with_expiration(&mut response, &session_series, &new_refresh_token)
                            }

                            let _ = self.try_refresh_session_series(&session_series, account_id, REFRESH_PAIR_EXPIRATION).await;
                        }
                    }

                    return Ok(response);
                },
                Err(ReAuthenticateSessionError::PotentialSessionTheft(account_id)) => self.mitigate_session_theft(account_id).await,
                _ => (),
            }
        }

        Err(ManageSessionError::AuthenticationFailed)
    }
}

#[derive(Debug, Error)]
pub enum ManageSessionError {
    #[error("セッションが存在しません")]
    NoSession,
    #[error("認証に失敗しました")]
    AuthenticationFailed,
}

#[cfg(test)]
mod tests {
    use std::{convert::Infallible, future::{ready, Ready}, str::FromStr, sync::LazyLock, task::{Context, Poll}};

    use http::{header::COOKIE, Request, Response};
    use tower::Service;

    use crate::{common::{email::address::Email, fallible::Fallible, profile::{account_id::AccountId, language::Language}, session::{cookie::{to_cookie_value, REFRESH_PAIR_COOKIE_KEY, SESSION_COOKIE_KEY}, refresh_pair_expiration::RefreshPairExpirationSeconds, refresh_token::RefreshToken, session_expiration::SessionExpirationSeconds, session_id::SessionId, session_series::SessionSeries}, unixtime::UnixtimeMillis}, middlewares::manage_session::dsl::{authenticate::{AuthenticateSession, AuthenticateSessionError}, extract_session_info::ExtractSessionInformation, mitigate_session_theft::{MitigateSessionTheft, MitigateSessionTheftError}, reauthenticate::{ReAuthenticateSession, ReAuthenticateSessionError}, refresh_session_series::{LastSessionSeriesRefreshedAt, RefreshSessionSeries, RefreshSessionSeriesError, SessionSeriesRefreshThereshold}, update_refresh_token::{UpdateRefreshToken, UpdateRefreshTokenError}, update_session::{UpdateSession, UpdateSessionError}}};

    use super::{ManageSession, ManageSessionError};

    static AUTHENTICATION_SUCCEEDED: LazyLock<SessionId> = LazyLock::new(SessionId::gen);
    static REAUTHENTICATION_SUCCEDED: LazyLock<(SessionSeries, RefreshToken)> = LazyLock::new(|| (SessionSeries::gen(), RefreshToken::gen()));

    struct MockManageSession;

    impl ManageSession for MockManageSession {}

    impl ExtractSessionInformation for MockManageSession {}

    impl AuthenticateSession for MockManageSession {
        async fn resolve_session_id_to_account_id(&self, session_id: &SessionId) -> Fallible<Option<AccountId>, AuthenticateSessionError> {
            if session_id == &*AUTHENTICATION_SUCCEEDED {
                Ok(Some(AccountId::gen()))
            } else {
                Ok(None)
            }
        }
    }

    impl ReAuthenticateSession for MockManageSession {
        async fn fetch_refresh_token_and_account_id(&self, session_series: &SessionSeries) -> Fallible<Option<(RefreshToken, AccountId)>, ReAuthenticateSessionError> {
            if session_series == &REAUTHENTICATION_SUCCEDED.0 {
                Ok(Some((REAUTHENTICATION_SUCCEDED.1.clone(), AccountId::gen())))
            } else {
                Ok(None)
            }
        }
    }

    impl UpdateSession for MockManageSession {
        async fn try_assign_new_session_id_with_expiration_if_unused(&self, _: &SessionId, _: AccountId, _: SessionExpirationSeconds) -> Fallible<(), UpdateSessionError> {
            Ok(())
        }
    }

    impl UpdateRefreshToken for MockManageSession {
        async fn assign_new_refresh_token_with_expiration(&self, _: &RefreshToken, _: &SessionSeries, _: AccountId, _: RefreshPairExpirationSeconds) -> Fallible<(), UpdateRefreshTokenError> {
            Ok(())
        }
    }

    const REFRESH_THERESHOLD: SessionSeriesRefreshThereshold = SessionSeriesRefreshThereshold::days(1);

    impl RefreshSessionSeries for MockManageSession {
        async fn fetch_last_session_series_refreshed_at(&self, _: &SessionSeries, _: AccountId) -> Fallible<LastSessionSeriesRefreshedAt, RefreshSessionSeriesError> {
            Ok(LastSessionSeriesRefreshedAt::new(UnixtimeMillis::now()))
        }

        fn refresh_thereshold() -> &'static SessionSeriesRefreshThereshold {
            &REFRESH_THERESHOLD
        }
    
        async fn refresh_session_series(&self, _: &SessionSeries, _: AccountId, _: RefreshPairExpirationSeconds) -> Fallible<(), RefreshSessionSeriesError> {
            Ok(())
        }
    }

    impl MitigateSessionTheft for MockManageSession {
        async fn fetch_email_and_language(&self, _: AccountId) -> Fallible<(Email, Language), MitigateSessionTheftError> {
            Ok((Email::from_str("email@example.com").unwrap(), Language::Japanese))
        }

        async fn send_security_notification(&self, _: &Email, _: Language) -> Fallible<(), MitigateSessionTheftError> {
            Ok(())
        }
    
        async fn purge_all_session_series(&self, _: AccountId) -> Fallible<(), MitigateSessionTheftError> {
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

        fn call(&mut self, _: Request<()>) -> Self::Future {
            ready(Ok(Response::new(())))
        }
    }
    
    async fn test_manage_session(session_cookie_key: &str, session_cookie_value: &str) -> Result<Response<()>, ManageSessionError> {
        let mut request = Request::new(());
        let header_value = format!("{}={}", session_cookie_key, session_cookie_value)
            .parse()
            .unwrap();
        request.headers_mut().insert(COOKIE, header_value);

        println!("{:?}", request);

        let manage_session = MockManageSession;

        manage_session.manage_session(&mut MockService, request).await
    }

    #[tokio::test]
    async fn authentication_succeeded() {
        let session_cookie_value = (*AUTHENTICATION_SUCCEEDED).to_string();
        let result = test_manage_session(SESSION_COOKIE_KEY, &session_cookie_value).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn reauthentication_succeeded() {
        let session_cookie_value = to_cookie_value(&REAUTHENTICATION_SUCCEDED.0, &REAUTHENTICATION_SUCCEDED.1);
        let result = test_manage_session(REFRESH_PAIR_COOKIE_KEY, &session_cookie_value).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn no_session() {
        let result = test_manage_session(SESSION_COOKIE_KEY, "invalid").await;
        assert!(matches!(result, Err(ManageSessionError::NoSession)));
    }
}