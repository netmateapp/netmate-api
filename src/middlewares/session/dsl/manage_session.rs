use std::convert::Infallible;

use http::{header::SET_COOKIE, HeaderName, HeaderValue, Request, Response};
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

        Err(ManageSessionError::InvalidSession(Self::clear_session_related_cookie_headers()))
    }

    fn session_expiration() -> &'static SessionExpirationSeconds;

    fn refresh_pair_expiration() -> &'static RefreshPairExpirationSeconds;
}

#[derive(Debug, Error)]
pub enum ManageSessionError {
    #[error("セッションが存在しません")]
    NoSession,
    #[error("無効なセッションです")]
    InvalidSession([(HeaderName, HeaderValue); 2]),
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