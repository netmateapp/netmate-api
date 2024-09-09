use std::convert::Infallible;

use http::{header::SET_COOKIE, Request, Response};
use tower::Service;

use crate::common::fallible::Fallible;

use super::{authenticate::AuthenticateSession, extract_session_info::ExtractSessionInformation, mitigate::MitigateSessionTheft, reauthenticate::{ReAuthenticateSession, ReAuthenticateSessionError}, set_cookie::SetSessionCookie, update_refresh_token::{RefreshPairExpirationSeconds, UpdateRefreshToken}, update_session::{SessionExpirationSeconds, UpdateSession}};

pub(crate) trait ManageSession {
    async fn manage_session<S, B>(&self, inner: &mut S, mut request: Request<B>) -> Fallible<S::Response, ManageSessionError>
    where
        Self: ExtractSessionInformation + SetSessionCookie + AuthenticateSession + ReAuthenticateSession + UpdateSession + UpdateRefreshToken + MitigateSessionTheft,
        S: Service<Request<B>, Error = Infallible, Response = Response<B>>,
    {
        let (session_id, pair) = Self::extract_session_information(&request);

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
                        // 新規セッションIDの発行に失敗した場合は、再認証が通常認証の代わりとなり、
                        // 本来インメモリキャッシュが負担すべき負荷がデータベースに流れる
                        match self.update_session(&account_id, Self::session_expiration()).await {
                            Ok(new_session_id) => Self::set_session_cookie_with_expiration(&mut response, &new_session_id),
                            _ => (),
                        }

                        // リフレッシュトークンの発行が失敗した場合は、現在のトークンを使用し続ける
                        // これはセキュリティリスクを多少増加させるが許容の範囲内である
                        match self.update_refresh_token(&session_series, &account_id, Self::refresh_token_expiration()).await {
                            Ok(new_refresh_token) => Self::set_refresh_pair_cookie_with_expiration(&mut response, &session_series, &new_refresh_token),
                            _ => (),
                        }
                    }

                    return Ok(response);
                },
                Err(ReAuthenticateSessionError::PotentialSessionTheft(account_id)) => self.mitigate_session_theft(&account_id).await,
                _ => (),
            }
        }

        Err(ManageSessionError::InvalidRequest)
    }

    fn session_expiration() -> &'static SessionExpirationSeconds;

    fn refresh_token_expiration() -> &'static RefreshPairExpirationSeconds;
}

pub enum ManageSessionError {
    InvalidRequest,
}
