use std::convert::Infallible;

use http::{header::SET_COOKIE, Request, Response};
use scylla::{frame::response::result::ColumnType, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use thiserror::Error;
use tower::Service;

use crate::common::fallible::Fallible;

use super::{authenticate::AuthenticateSession, extract_session_info::ExtractSessionInformation, mitigate_session_theft::MitigateSessionTheft, reauthenticate::{ReAuthenticateSession, ReAuthenticateSessionError}, refresh_session_series::RefreshSessionSeries, set_cookie::SetSessionCookie, update_refresh_token::UpdateRefreshToken, update_session::UpdateSession};

pub(crate) trait ManageSession {
    async fn manage_session<S, B>(&self, inner: &mut S, mut request: Request<B>) -> Fallible<S::Response, ManageSessionError>
    where
        Self: ExtractSessionInformation + SetSessionCookie + AuthenticateSession + ReAuthenticateSession + UpdateSession + UpdateRefreshToken + RefreshSessionSeries + MitigateSessionTheft,
        S: Service<Request<B>, Error = Infallible, Response = Response<B>>,
    {
        let (session_id, pair) = Self::extract_session_information(&request);

        if session_id.is_none() && pair.is_none() {
            return Err(ManageSessionError::NoSession);
        }

        if let Some(session_id) = session_id {
            match self.authenticate_session(&session_id).await {
                Ok(account_id) => {
                    request.extensions_mut().insert(account_id);

                    // `Error`は`Infallible`で起こり得ないので`unwrap()`で問題ない
                    let mut response = inner.call(request).await.unwrap();

                    // パスワード変更やログアウトによるSet-Cookieヘッダが無い場合のみセッションを延長
                    if !response.headers().contains_key(SET_COOKIE) {
                        Self::refresh_session_cookie_expiration(&mut response, &session_id);
                    }
                    
                    return Ok(response)
                },
                _ => (),
            }
        }
    
        if let Some((session_series, refresh_token)) = pair {
            match self.reauthenticate_session(&session_series, refresh_token).await {
                Ok(account_id) => {
                    request.extensions_mut().insert(account_id.clone());

                    // `Error`は`Infallible`で起こり得ないので`unwrap()`で問題ない
                    let mut response = inner.call(request).await.unwrap();
            
                    // パスワード変更やログアウトによるSet-Cookieヘッダが無い場合のみセッションを延長
                    if !response.headers().contains_key(SET_COOKIE) {
                        // セッションIDの更新に成功した場合のみに限定することで、
                        // 基本的に最低30分は間隔を空けて更新処理を行うようにし負荷を抑える
                        // ※セッションIDを破棄して送信されるリクエストへの耐性は無い
                        if let Ok(new_session_id) = self.update_session(&account_id, Self::session_expiration()).await {
                            Self::set_session_cookie_with_expiration(&mut response, &new_session_id);

                            // リフレッシュトークンの発行が失敗した場合は、現在のトークンを使用し続ける
                            // これはセキュリティリスクを多少増加させるが許容の範囲内である
                            match self.update_refresh_token(&session_series, &account_id, Self::refresh_pair_expiration()).await {
                                Ok(new_refresh_token) => Self::set_refresh_pair_cookie_with_expiration(&mut response, &session_series, &new_refresh_token),
                                _ => (),
                            }

                            let _ = self.try_refresh_session_series(&session_series, &account_id, Self::refresh_pair_expiration()).await;
                        }
                    }

                    return Ok(response);
                },
                Err(ReAuthenticateSessionError::PotentialSessionTheft(account_id)) => self.mitigate_session_theft(&account_id).await,
                _ => (),
            }
        }

        Err(ManageSessionError::AuthenticationFailed)
    }

    fn session_expiration() -> &'static SessionExpirationSeconds;

    fn refresh_pair_expiration() -> &'static RefreshPairExpirationSeconds;
}

#[derive(Debug, Error)]
pub enum ManageSessionError {
    #[error("セッションが存在しません")]
    NoSession,
    #[error("認証に失敗しました")]
    AuthenticationFailed,
}

pub struct SessionExpirationSeconds(u32);

impl SessionExpirationSeconds {
    pub const fn new(seconds: u32) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RefreshPairExpirationSeconds(u32);

impl RefreshPairExpirationSeconds {
    pub const fn new(seconds: u32) -> Self {
        Self(seconds)
    }

    pub fn as_secs(&self) -> u32 {
        self.0
    }
}

impl From<RefreshPairExpirationSeconds> for i32 {
    fn from(expiration: RefreshPairExpirationSeconds) -> Self {
        expiration.0 as i32
    }
}

impl SerializeValue for RefreshPairExpirationSeconds {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        (self.0 as i32).serialize(typ, writer)
    }
}

#[cfg(test)]
mod tests {
    use std::{convert::Infallible, future::{ready, Ready}, str::FromStr, sync::LazyLock, task::{Context, Poll}};

    use http::{header::COOKIE, Request, Response};
    use tower::Service;

    use crate::{common::{email::address::Email, fallible::Fallible, id::{uuid7::Uuid7, AccountId}, language::Language, session::value::{to_cookie_value, RefreshToken, SessionId, SessionSeries, REFRESH_PAIR_COOKIE_KEY, SESSION_COOKIE_KEY}, unixtime::UnixtimeMillis}, middlewares::manage_session::dsl::{authenticate::{AuthenticateSession, AuthenticateSessionError}, extract_session_info::ExtractSessionInformation, mitigate_session_theft::{MitigateSessionTheft, MitigateSessionTheftError}, reauthenticate::{ReAuthenticateSession, ReAuthenticateSessionError}, refresh_session_series::{LastSessionSeriesRefreshedAt, RefreshSessionSeries, RefreshSessionSeriesError, SessionSeriesRefreshThereshold}, set_cookie::SetSessionCookie, update_refresh_token::{UpdateRefreshToken, UpdateRefreshTokenError}, update_session::{UpdateSession, UpdateSessionError}}};

    use super::{ManageSession, ManageSessionError, RefreshPairExpirationSeconds, SessionExpirationSeconds};

    static AUTHENTICATION_SUCCEEDED: LazyLock<SessionId> = LazyLock::new(|| SessionId::gen());
    static REAUTHENTICATION_SUCCEDED: LazyLock<(SessionSeries, RefreshToken)> = LazyLock::new(|| (SessionSeries::gen(), RefreshToken::gen()));

    const SESSION_EXPIRATION: SessionExpirationSeconds = SessionExpirationSeconds::new(1800);
    const REFRESH_PAIR_EXPIRATION: RefreshPairExpirationSeconds = RefreshPairExpirationSeconds::new(2592000);

    struct MockManageSession;

    impl ManageSession for MockManageSession {
        fn session_expiration() -> &'static SessionExpirationSeconds {
            &SESSION_EXPIRATION
        }

        fn refresh_pair_expiration() -> &'static RefreshPairExpirationSeconds {
            &REFRESH_PAIR_EXPIRATION
        }
    }

    impl ExtractSessionInformation for MockManageSession {}

    impl SetSessionCookie for MockManageSession {}

    impl AuthenticateSession for MockManageSession {
        async fn resolve_session_id_to_account_id(&self, session_id: &SessionId) -> Fallible<Option<AccountId>, AuthenticateSessionError> {
            if session_id == &*AUTHENTICATION_SUCCEEDED {
                Ok(Some(AccountId::new(Uuid7::now())))
            } else {
                Ok(None)
            }
        }
    }

    impl ReAuthenticateSession for MockManageSession {
        async fn fetch_refresh_token_and_account_id(&self, session_series: &SessionSeries) -> Fallible<Option<(RefreshToken, AccountId)>, ReAuthenticateSessionError> {
            if session_series == &(*REAUTHENTICATION_SUCCEDED).0 {
                Ok(Some((REAUTHENTICATION_SUCCEDED.1.clone(), AccountId::new(Uuid7::now()))))
            } else {
                Ok(None)
            }
        }
    }

    impl UpdateSession for MockManageSession {
        async fn try_assign_new_session_id_with_expiration_if_unused(&self, _: &SessionId, _: &AccountId, _: &SessionExpirationSeconds) -> Fallible<(), UpdateSessionError> {
            Ok(())
        }
    }

    impl UpdateRefreshToken for MockManageSession {
        async fn assign_new_refresh_token_with_expiration(&self, _: &RefreshToken, _: &SessionSeries, _: &AccountId, _: &RefreshPairExpirationSeconds) -> Fallible<(), UpdateRefreshTokenError> {
            Ok(())
        }
    }

    const REFRESH_THERESHOLD: SessionSeriesRefreshThereshold = SessionSeriesRefreshThereshold::days(1);

    impl RefreshSessionSeries for MockManageSession {
        async fn fetch_last_session_series_refreshed_at(&self, _: &SessionSeries, _: &AccountId) -> Fallible<LastSessionSeriesRefreshedAt, RefreshSessionSeriesError> {
            Ok(LastSessionSeriesRefreshedAt::new(UnixtimeMillis::now()))
        }

        fn refresh_thereshold() -> &'static SessionSeriesRefreshThereshold {
            &REFRESH_THERESHOLD
        }
    
        async fn refresh_session_series(&self, _: &SessionSeries, _: &AccountId, _: &RefreshPairExpirationSeconds) -> Fallible<(), RefreshSessionSeriesError> {
            Ok(())
        }
    }

    impl MitigateSessionTheft for MockManageSession {
        async fn fetch_email_and_language(&self, _: &AccountId) -> Fallible<(Email, Language), MitigateSessionTheftError> {
            Ok((Email::from_str("email@example.com").unwrap(), Language::Japanese))
        }

        async fn send_security_notification(&self, _: &Email, _: &Language) -> Fallible<(), MitigateSessionTheftError> {
            Ok(())
        }
    
        async fn purge_all_session_series(&self, _: &AccountId) -> Fallible<(), MitigateSessionTheftError> {
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
        let session_cookie_value = to_cookie_value(&(*REAUTHENTICATION_SUCCEDED).0, &(*REAUTHENTICATION_SUCCEDED).1);
        let result = test_manage_session(REFRESH_PAIR_COOKIE_KEY, &session_cookie_value).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn no_session() {
        let result = test_manage_session(SESSION_COOKIE_KEY, "invalid").await;
        assert!(matches!(result, Err(ManageSessionError::NoSession)));
    }
}