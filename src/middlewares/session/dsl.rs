        /*
        処理のパターンは5通り(S: セッション管理識別子, L: ログイン識別子)
        1. S (通常のセッション認証、これが最も多い)
        2. None/Fail(S) -> L (セッションの更新、次に多い)
        3. None/Fail(S) -> Fail(L) (セッション削除後/期限切れ後の場合、まれにある)
        4. Fail(S) -> None(L) (普通はない、クライアント側でユーザーが何らかの操作を行っている可能性がある)
        5. None(S) -> None(L) (UIからは送れないはず、UI外でエンドポイントを叩いている可能性が高い)
*/

use std::{convert::Infallible, time::{SystemTime, UNIX_EPOCH}};

use http::{header::SET_COOKIE, Extensions, HeaderMap, HeaderName, Request, Response};
use thiserror::Error;
use tower::Service;

use crate::common::{fallible::Fallible, id::AccountId, session::value::{LoginId, LoginSeriesId, LoginToken, SessionManagementId}};

pub(crate) trait ManageSession {
    async fn manage_session<S, B>(&self, mut inner: S, mut req: Request<B>) -> Fallible<Response<B>, ManageSessionError>
    where
        S: Service<Request<B>, Error = Infallible, Response = Response<B>>,
    {
        let (maybe_session_management_id, maybe_login_id) = Self::extract_session_ids(req.headers());

        if let Some(session_management_id) = maybe_session_management_id {
            if let Some(account_id) = self.resolve(&session_management_id).await? {
                Self::insert_account_id(req.extensions_mut(), &account_id);

                // `Error`は`Infallible`で起こり得ないので`unwrap()`で問題ない
                let mut response = inner.call(req).await.unwrap();

                // パスワード変更やログアウトによるSet-Cookieヘッダが無い場合のみセッションを延長
                if !response.headers().contains_key(SET_COOKIE) {
                    Self::reset_session_timeout(response.headers_mut(), &session_management_id);
                }
                return Ok(response);
            }
        }

        if let Some(login_id) = maybe_login_id {
            let (maybe_login_token, maybe_account_id) = self.get_login_token_and_account_id(login_id.series_id()).await?;
            
            if let (Some(login_token), Some(account_id)) = (maybe_login_token, maybe_account_id) {
                if login_id.token().value().value() == login_token.value().value() {
                    Self::insert_account_id(req.extensions_mut(), &account_id);

                    // `Error`は`Infallible`で起こり得ないので`unwrap()`で問題ない
                    let mut response = inner.call(req).await.unwrap();

                    // パスワード変更やログアウトによるSet-Cookieヘッダが無い場合のみセッションを延長
                    if !response.headers().contains_key(SET_COOKIE) {
                        // 新しい識別子やトークンの発行に失敗した場合は、エラーを無視してセッションを継続する
                        // これによる影響はセキュリティリスクの微小な増加のみである
                        let new_session_management_id = self.issue_new_session_management_id().await;
                        let new_login_token = self.issue_new_login_token().await;

                        if let (Ok(new_session_management_id), Ok(new_login_token)) = (new_session_management_id, new_login_token) {
                            Self::set_new_session_management_id(response.headers_mut(), &new_session_management_id);
                            Self::set_new_login_token(response.headers_mut(), &login_id.series_id(), &new_login_token);

                            let series_id_update_at = self.series_id_update_at(&account_id).await?;

                            let now = SystemTime::now().duration_since(UNIX_EPOCH);
                            if let Ok(now) = now {
                                if now.as_secs() - series_id_update_at.value() > 30 * 24 * 60 * 60 {
                                    // 既存のシリーズIDの有効期限を延長する
                                    self.extend_series_id_expiration(login_id.series_id()).await?;
                                }
                            }
                        }
                    }
                    return Ok(response);
                } else {
                    // セッション識別子が窃取された可能性

                    return Err(ManageSessionError::InvalidSession([
                        Self::clear_session_management_id_header(),
                        Self::clear_login_id_header()
                    ]));
                }
            } else {
                return Err(ManageSessionError::InvalidSession([
                    Self::clear_session_management_id_header(),
                    Self::clear_login_id_header()
                ]));
            }
        }

        Err(ManageSessionError::NoSession)
    }

    fn extract_session_ids(headers: &HeaderMap) -> (Option<SessionManagementId>, Option<LoginId>);

    async fn resolve(&self, session_management_id: &SessionManagementId) -> Fallible<Option<AccountId>, ManageSessionError>;

    fn insert_account_id(extensions: &mut Extensions, account_id: &AccountId);

    fn reset_session_timeout(headers: &mut HeaderMap, session_management_id: &SessionManagementId);

    async fn get_login_token_and_account_id(&self, series_id: &LoginSeriesId) -> Fallible<(Option<LoginToken>, Option<AccountId>), ManageSessionError>;

    async fn issue_new_session_management_id(&self) -> Fallible<SessionManagementId, ManageSessionError>;

    async fn issue_new_login_token(&self) -> Fallible<LoginToken, ManageSessionError>;

    async fn series_id_update_at(&self, account_id: &AccountId) -> Fallible<UnixtimeSeconds, ManageSessionError>;

    async fn extend_series_id_expiration(&self, series_id: &LoginSeriesId) -> Fallible<(), ManageSessionError>;

    fn set_new_session_management_id(headers: &mut HeaderMap, new_session_management_id: &SessionManagementId);

    fn set_new_login_token(headers: &mut HeaderMap, series_id: &LoginSeriesId, new_login_token: &LoginToken);

    fn clear_session_management_id_header() -> (HeaderName, &'static str);

    fn clear_login_id_header() -> (HeaderName, &'static str);
}

#[derive(Debug, Error)]
pub enum ManageSessionError {
    #[error("セッションがありません")]
    NoSession,
    #[error("無効なセッションです")]
    InvalidSession([(HeaderName, &'static str); 2])
}

pub struct UnixtimeSeconds(u64);

impl UnixtimeSeconds {
    pub fn new(unixtime_seconds: u64) -> Self {
        Self(unixtime_seconds)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}
