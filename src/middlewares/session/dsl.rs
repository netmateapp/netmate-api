use std::{convert::Infallible, time::{SystemTime, UNIX_EPOCH}};

use http::{header::SET_COOKIE, Extensions, HeaderMap, HeaderName, Request, Response};
use thiserror::Error;
use tower::Service;
use tracing::info;

use crate::common::{fallible::Fallible, id::AccountId, session::value::{LoginId, LoginSeriesId, LoginToken, SessionManagementId}};

pub(crate) trait ManageSession {
    /*
        処理のパターンは5通り(S: セッション管理識別子, L: ログイン識別子)
        1. S (通常のセッション認証、これが最も多い)
        2. None/Fail(S) -> L (セッションの更新、次に多い)
        3. None/Fail(S) -> Fail(L) (セッション削除後/期限切れ後の場合、まれにある)
        4. Fail(S) -> None(L) (普通はない、クライアント側でユーザーが何らかの操作を行っている可能性がある)
        5. None(S) -> None(L) (UIからは送れないはず、UI外でエンドポイントを叩いている可能性が高い)
    */
    async fn manage_session<S, B>(&self, mut inner: S, mut req: Request<B>) -> Fallible<Response<B>, ManageSessionError>
    where
        S: Service<Request<B>, Error = Infallible, Response = Response<B>>,
    {
        let (maybe_session_management_id, maybe_login_id) = Self::extract_session_ids(req.headers());

        // ここで戻ることは基本的にない
        // ルート設定が誤っているか、クライアントが不適切にリクエストをしているかのどちらかである
        if maybe_session_management_id.is_none() && maybe_login_id.is_none() {
            return Err(ManageSessionError::NoSession);
        }

        // 通常のセッション識別子からアカウント識別子の取得を試みる
        if let Some(session_management_id) = maybe_session_management_id {
            if let Some(account_id) = self.resolve(&session_management_id).await? {
                Self::insert_account_id(req.extensions_mut(), account_id.clone());

                // `Error`は`Infallible`で起こり得ないので`unwrap()`で問題ない
                let mut response = inner.call(req).await.unwrap();

                // パスワード変更やログアウトによるSet-Cookieヘッダが無い場合のみセッションを延長
                if Self::can_set_cookie_in_response_header(response.headers()) {
                    Self::reset_session_timeout(response.headers_mut(), &session_management_id);
                }
                return Ok(response);
            }
        }

        // 系列識別子からセッション識別子の生成とアカウント識別子の取得を試みる
        if let Some(login_id) = maybe_login_id {
            if let (Some(login_token), Some(account_id)) = self.get_login_token_and_account_id(login_id.series_id()).await? {
                if Self::is_same_token(&login_id.token(), &login_token) {
                    Self::insert_account_id(req.extensions_mut(), account_id.clone());

                    // `Error`は`Infallible`で起こり得ないので`unwrap()`で問題ない
                    let mut response = inner.call(req).await.unwrap();

                    // パスワード変更やログアウトによるSet-Cookieヘッダが無い場合のみセッションを延長
                    if Self::can_set_cookie_in_response_header(response.headers()) {
                        self.rotate_session_ids(response.headers_mut(), &account_id, &login_id.series_id()).await;
                    }

                    return Ok(response);
                } else {
                    // セッション識別子が窃取された可能性

                    let is_email_sent = self.send_security_notifications_email(&account_id)
                        .await
                        .is_ok();

                    let can_print_series_id = self.delete_all_sessions(&account_id)
                        .await
                        .is_ok();

                    // 全てのセッションが削除されたため、ログに出力して問題ない
                    info!(
                        account_id = %account_id.value().value(),
                        series_id = if can_print_series_id { Some(login_id.series_id().value().value()) } else { None },
                        is_email_sent = is_email_sent,
                        "セッション識別子が盗用された可能性を検知しました"
                    );
                }
            }
        }

        // 無効なセッション識別子であるため、削除指令を送信する
        Err(ManageSessionError::InvalidSession([
            Self::clear_session_management_id_header(),
            Self::clear_login_id_header()
        ]))
    }

    fn extract_session_ids(headers: &HeaderMap) -> (Option<SessionManagementId>, Option<LoginId>);

    async fn resolve(&self, session_management_id: &SessionManagementId) -> Fallible<Option<AccountId>, ManageSessionError>;

    fn insert_account_id(extensions: &mut Extensions, account_id: AccountId) {
        extensions.insert(account_id);
    }

    fn can_set_cookie_in_response_header(headers: &HeaderMap) -> bool {
        !headers.contains_key(SET_COOKIE)
    }

    fn reset_session_timeout(headers: &mut HeaderMap, session_management_id: &SessionManagementId);

    async fn get_login_token_and_account_id(&self, series_id: &LoginSeriesId) -> Fallible<(Option<LoginToken>, Option<AccountId>), ManageSessionError>;

    fn is_same_token(request_token: &LoginToken, registered_token: &LoginToken) -> bool {
        request_token.value().value() == registered_token.value().value()
    }

    async fn rotate_session_ids(&self, headers: &mut HeaderMap, account_id: &AccountId, series_id: &LoginSeriesId) -> Fallible<(), ManageSessionError> {
        // 新しい識別子やトークンの登録に失敗した場合は、エラーを無視してセッションを継続する
        // これによる影響はセキュリティリスクの微小な増加のみである
        // 登録に成功した場合は必ずヘッダーに付加する
        let new_session_management_id = SessionManagementId::gen();
        self.register_new_session_management_id_with_account_id(&new_session_management_id, account_id).await?;
        Self::set_new_session_management_id_in_header(headers, &new_session_management_id);

        // 新しいログイントークンの登録に失敗した場合は、現在のトークンを使い続けることになる
        // トークンの窃取と誤判定されることはない
        let new_login_token = LoginToken::gen();
        self.register_new_login_id_with_account_id(series_id, &new_login_token, account_id).await?;
        Self::set_new_login_token_in_header(headers, series_id, &new_login_token);

        self.try_extend_series_id_expiration(&account_id, series_id).await
    }

    async fn register_new_session_management_id_with_account_id(&self, new_session_management_id: &SessionManagementId, account_id: &AccountId) -> Fallible<(), ManageSessionError>;

    async fn register_new_login_id_with_account_id(&self, login_series_id: &LoginSeriesId, new_login_token: &LoginToken, account_id: &AccountId) -> Fallible<(), ManageSessionError>;

    async fn try_extend_series_id_expiration(&self, account_id: &AccountId, series_id: &LoginSeriesId) -> Fallible<(), ManageSessionError> {
        // `series_id_update_at`は実際にはDBアクセスとなるため、
        // 正常にセッション管理識別子が発行されている時 = 次のアクセスが最短でも30分後である高い保証がある場合のみ処理する
        // ユーザーが意図的にセッション管理クッキーを削除した場合は、30分以内にもアクセスされる可能性がある
        // その点は`series_id_update_at`内でレートリミットを設け対策する
        let should_extend = self.last_series_id_expiration_update_time(account_id, series_id)
            .await
            .and_then(|t| Self::should_extend_series_id_expiration(&t))?;

        if should_extend {
            // 既存のシリーズIDの有効期限を延長する
            self.extend_series_id_expiration(series_id).await
        } else {
            Ok(())
        }
    }

    async fn last_series_id_expiration_update_time(&self, account_id: &AccountId, series_id: &LoginSeriesId) -> Fallible<UnixtimeSeconds, ManageSessionError>;

    fn should_extend_series_id_expiration(last_series_id_expiration_update_time: &UnixtimeSeconds) -> Fallible<bool, ManageSessionError> {
        let current_unixtime = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .map_err(|e| ManageSessionError::CheckSeriesIdExpirationExtendabilityFailed(e.into()))?;

        const SESSION_EXTENSION_THRESHOLD: u64 = 30 * 24 * 60 * 60;
        
        Ok(current_unixtime - last_series_id_expiration_update_time.value() > SESSION_EXTENSION_THRESHOLD)
    }

    async fn extend_series_id_expiration(&self, series_id: &LoginSeriesId) -> Fallible<(), ManageSessionError>;

    fn set_new_session_management_id_in_header(headers: &mut HeaderMap, new_session_management_id: &SessionManagementId);

    fn set_new_login_token_in_header(headers: &mut HeaderMap, series_id: &LoginSeriesId, new_login_token: &LoginToken);

    async fn delete_all_sessions(&self, account_id: &AccountId) -> Fallible<(), ManageSessionError>;

    async fn send_security_notifications_email(&self, account_id: &AccountId) -> Fallible<(), ManageSessionError>;

    fn clear_session_management_id_header() -> (HeaderName, &'static str);

    fn clear_login_id_header() -> (HeaderName, &'static str) ;
}

#[derive(Debug, Error)]
pub enum ManageSessionError {
    #[error("セッションがありません")]
    NoSession,
    #[error("無効なセッションです")]
    InvalidSession([(HeaderName, &'static str); 2]),
    #[error("系列識別子の期限延長可能性の確認に失敗しました")]
    CheckSeriesIdExpirationExtendabilityFailed(#[source] anyhow::Error),
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
