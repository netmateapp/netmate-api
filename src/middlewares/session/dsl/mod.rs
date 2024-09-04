use std::convert::Infallible;

use extract_session_ids::extract_session_ids;
use http::{HeaderMap, Request, Response};
use misc::{can_set_cookie_in_response_header, insert_account_id, is_same_token, should_extend_series_id_expiration, UnixtimeSeconds};
use set_cookie::{clear_session_ids_in_response_header, reset_session_timeout, set_new_login_token_in_header, set_new_session_management_id_in_header};
use thiserror::Error;
use tower::Service;
use tracing::info;

use crate::common::{fallible::Fallible, id::AccountId, session::value::{LoginSeriesId, LoginToken, SessionManagementId}};

mod extract_session_ids;
mod misc;
mod set_cookie;

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
        let (maybe_session_management_id, maybe_login_id) = extract_session_ids(req.headers());

        // ここで戻ることは基本的にない
        // ルート設定が誤っているか、クライアントが不適切にリクエストをしているかのどちらかである
        if maybe_session_management_id.is_none() && maybe_login_id.is_none() {
            return Err(ManageSessionError::NoSession);
        }

        // 通常のセッション識別子からアカウント識別子の取得を試みる
        if let Some(session_management_id) = maybe_session_management_id {
            if let Some(account_id) = self.resolve(&session_management_id).await? {
                insert_account_id(req.extensions_mut(), account_id.clone());

                // `Error`は`Infallible`で起こり得ないので`unwrap()`で問題ない
                let mut response = inner.call(req).await.unwrap();

                // パスワード変更やログアウトによるSet-Cookieヘッダが無い場合のみセッションを延長
                if can_set_cookie_in_response_header(response.headers()) {
                    reset_session_timeout(response.headers_mut(), &session_management_id);
                }
                return Ok(response);
            }
        }

        // 系列識別子から新しいセッション識別子の生成とアカウント識別子の取得を試みる
        if let Some(login_id) = maybe_login_id {
            if let (Some(login_token), Some(account_id)) = self.get_login_token_and_account_id(login_id.series_id()).await? {
                if is_same_token(&login_id.token(), &login_token) {
                    insert_account_id(req.extensions_mut(), account_id.clone());

                    // `Error`は`Infallible`で起こり得ないので`unwrap()`で問題ない
                    let mut response = inner.call(req).await.unwrap();

                    // パスワード変更やログアウトによるSet-Cookieヘッダが無い場合のみセッションを延長
                    if can_set_cookie_in_response_header(response.headers()) {
                        // 誤ってエラー委譲でレスポンスを返さず戻ることがないよう型を明示
                        let _: Result<_, _> = self.rotate_session_ids(response.headers_mut(), &account_id, &login_id.series_id()).await;
                    }

                    return Ok(response);
                } else {
                    // この分岐に到達した場合、ログイン識別子が盗用された可能性がある

                    let is_email_sent = self.send_security_notifications_email(&account_id)
                        .await
                        .is_ok();

                    match self.delete_all_sessions(&account_id).await {
                        // セッションは削除されているため、ログに出力しても問題ない
                        Ok(_) => info!(
                            account_id = %account_id.value().value(),
                            series_id = login_id.series_id().value().value(),
                            is_email_sent = is_email_sent,
                            "セッション識別子が盗用された可能性を検知したため、当該アカウントの全セッションを削除しました"
                        ),
                        Err(_) => info!(
                            account_id = %account_id.value().value(),
                            is_email_sent = is_email_sent,
                            "セッション識別子が盗用された可能性を検知しましたが、当該アカウントの全セッションの削除に失敗しました"
                        )
                    }                    
                }
            }
        }

        // 無効なセッション識別子であるため、削除指令を送信する
        Err(ManageSessionError::InvalidSession(clear_session_ids_in_response_header()))
    }

    async fn resolve(&self, session_management_id: &SessionManagementId) -> Fallible<Option<AccountId>, ManageSessionError>;

    async fn get_login_token_and_account_id(&self, series_id: &LoginSeriesId) -> Fallible<(Option<LoginToken>, Option<AccountId>), ManageSessionError>;

    async fn rotate_session_ids(&self, response_headers: &mut HeaderMap, account_id: &AccountId, series_id: &LoginSeriesId) -> Fallible<(), ManageSessionError> {
        // 新しい識別子やトークンの登録に失敗した場合は、エラーを無視してセッションを継続する
        // これによる影響はセキュリティリスクの微小な増加のみである
        // 登録に成功した場合は必ずヘッダーに付加する
        let new_session_management_id = SessionManagementId::gen();
        self.register_new_session_management_id_with_account_id(&new_session_management_id, account_id).await?;
        set_new_session_management_id_in_header(response_headers, &new_session_management_id);

        // 新しいログイントークンの登録に失敗した場合は、現在のトークンを使い続けることになる
        // トークンの窃取と誤判定されることはない
        let new_login_token = LoginToken::gen();
        self.register_new_login_id_with_account_id(series_id, &new_login_token, account_id).await?;
        set_new_login_token_in_header(response_headers, series_id, &new_login_token);

        self.try_extend_series_id_expiration(&account_id, series_id).await
    }

    async fn register_new_session_management_id_with_account_id(&self, new_session_management_id: &SessionManagementId, account_id: &AccountId) -> Fallible<(), ManageSessionError>;

    async fn register_new_login_id_with_account_id(&self, login_series_id: &LoginSeriesId, new_login_token: &LoginToken, account_id: &AccountId) -> Fallible<(), ManageSessionError>;

    async fn try_extend_series_id_expiration(&self, account_id: &AccountId, series_id: &LoginSeriesId) -> Fallible<(), ManageSessionError> {
        // `series_id_update_at`は実際にはDBアクセスとなるため、
        // 正常にセッション管理識別子が発行されている時 = 次のアクセスが最短でも30分後である高い保証がある場合のみ処理する
        // ユーザーが意図的にセッション管理クッキーを削除した場合は、30分以内にもアクセスされる可能性がある
        // その点は`series_id_update_at`内でレートリミットを設け対策する
        let should_extend = self.get_last_series_id_extension_time(account_id, series_id)
            .await
            .and_then(|t| should_extend_series_id_expiration(&t))?;

        if should_extend {
            // 既存のシリーズIDの有効期限を延長する
            self.extend_series_id_expiration(series_id).await
        } else {
            Ok(())
        }
    }

    async fn get_last_series_id_extension_time(&self, account_id: &AccountId, series_id: &LoginSeriesId) -> Fallible<UnixtimeSeconds, ManageSessionError>;

    async fn extend_series_id_expiration(&self, series_id: &LoginSeriesId) -> Fallible<(), ManageSessionError>;

    async fn delete_all_sessions(&self, account_id: &AccountId) -> Fallible<(), ManageSessionError>;

    async fn send_security_notifications_email(&self, account_id: &AccountId) -> Fallible<(), ManageSessionError>;
}

#[derive(Debug, Error)]
pub enum ManageSessionError {
    #[error("セッションがありません")]
    NoSession,
    #[error("アカウント識別子への解決に失敗しました")]
    ResolveFailed(#[source] anyhow::Error),
    #[error("ログイントーク及びアカウント識別子の取得に失敗しました")]
    GetLoginTokenAndAccountIdFailed(#[source] anyhow::Error),
    #[error("新しいセッション管理識別子の登録に失敗しました")]
    RegisteredNewSessionManagementIdFailed(#[source] anyhow::Error),
    #[error("新しいログイン識別子の登録に失敗しました")]
    RegisteredNewLoginIdFailed(#[source] anyhow::Error),
    #[error("系列識別子の期限延長可能性の確認に失敗しました")]
    CheckSeriesIdExpirationExtendabilityFailed(#[source] anyhow::Error),
    #[error("無効なセッションです")]
    InvalidSession(HeaderMap),
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::str::FromStr;
    use std::sync::LazyLock;
    use std::task::{Context, Poll};
    use http::header::COOKIE;
    use http::{HeaderValue, Request, Response};
    use tower::Service;

    use crate::common::fallible::Fallible;
    use crate::common::id::AccountId;
    use crate::common::session::value::{to_cookie_value, LoginId, LoginSeriesId, LoginToken, SessionManagementId, LOGIN_COOKIE_KEY, SESSION_MANAGEMENT_COOKIE_KEY};
    use super::{misc::UnixtimeSeconds, ManageSession, ManageSessionError};

    /*
        1. S (通常のセッション認証、これが最も多い)
        2. 空/無登録(S) -> L (セッションの更新、次に多い)
        3. 空/無登録(S) -> 無登録(L) (セッション削除後/期限切れ後の場合、まれにある)
        4. 無登録(S) -> 空(L) (普通はない、クライアント側でユーザーが何らかの操作を行っている可能性がある)
        5. 空/無登録(S) -> 不一致(L) (セッション識別子が盗用された可能性が高い)
        6. 空(S) -> 空(L) (UIからは送れないはず、UI外でエンドポイントを叩いている可能性が高い)
     */

    static CASE_1: LazyLock<SessionManagementId> = LazyLock::new(|| SessionManagementId::gen());
    static CASE_2: LazyLock<LoginId> = LazyLock::new(|| LoginId::new(LoginSeriesId::gen(), LoginToken::gen()));
    static CASE_3: LazyLock<LoginId> = LazyLock::new(|| LoginId::new(LoginSeriesId::gen(), LoginToken::gen()));
    static CASE_4: LazyLock<SessionManagementId> = LazyLock::new(|| SessionManagementId::gen());
    static CASE_5: LazyLock<LoginId> = LazyLock::new(|| LoginId::new(LoginSeriesId::gen(), LoginToken::gen()));

    struct MockManageSession;

    impl ManageSession for MockManageSession {
        async fn resolve(&self, session_management_id: &SessionManagementId) -> Fallible<Option<AccountId>, ManageSessionError> {
            if session_management_id == &*CASE_1 {
                Ok(Some(AccountId::gen()))
            } else {
                Ok(None)
            }
        }

        async fn get_login_token_and_account_id(&self, series_id: &LoginSeriesId) -> Fallible<(Option<LoginToken>, Option<AccountId>), ManageSessionError> {
            if series_id == &*CASE_2.series_id() {
                Ok((Some(LoginToken::from_str(*&CASE_2.token().value().value()).unwrap()), Some(AccountId::gen())))
            } else if series_id == &*CASE_5.series_id() {
                Ok((Some(LoginToken::gen()), Some(AccountId::gen())))
            } else {
                Ok((None, None))
            }
        }

        async fn register_new_session_management_id_with_account_id(&self, _: &SessionManagementId, _: &AccountId) -> Fallible<(), ManageSessionError> {
            Ok(())
        }

        async fn register_new_login_id_with_account_id(&self, _: &LoginSeriesId, _: &LoginToken, _: &AccountId) -> Fallible<(), ManageSessionError> {
            Ok(())
        }

        async fn get_last_series_id_extension_time(&self, _: &AccountId, _: &LoginSeriesId) -> Fallible<UnixtimeSeconds, ManageSessionError> {
            Ok(UnixtimeSeconds::new(0))
        }

        async fn extend_series_id_expiration(&self, _: &LoginSeriesId) -> Fallible<(), ManageSessionError> {
            Ok(())
        }

        async fn delete_all_sessions(&self, _: &AccountId) -> Fallible<(), ManageSessionError> {
            Ok(())
        }

        async fn send_security_notifications_email(&self, _: &AccountId) -> Fallible<(), ManageSessionError> {
            Ok(())
        }
    }

    struct MockInnerService;

    impl Service<Request<()>> for MockInnerService {
        type Response = Response<()>;
        type Error = Infallible;
        type Future = std::future::Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _: Request<()>) -> Self::Future {
            std::future::ready(Ok(Response::new(())))
        }
    }

    async fn test_case(session_management_id: Option<&SessionManagementId>, login_id: Option<&LoginId>) -> Result<Response<()>, ManageSessionError> {
        let mut request = Request::new(());
        if let (Some(session_management_id), Some(login_id)) = (session_management_id, login_id) {
            let cookie_str = format!("{}={}; {}={}", SESSION_MANAGEMENT_COOKIE_KEY, session_management_id.value().value(), LOGIN_COOKIE_KEY, to_cookie_value(login_id.series_id(), login_id.token()));
            request.headers_mut().insert(COOKIE, HeaderValue::from_str(&cookie_str).unwrap());
        } else if let Some(session_management_id) = session_management_id {
            let cookie_str = format!("{}={}", SESSION_MANAGEMENT_COOKIE_KEY, session_management_id.value().value());
            request.headers_mut().insert(COOKIE, HeaderValue::from_str(&cookie_str).unwrap());
        } else if let Some(login_id) = login_id {
            let cookie_str = format!("{}={}", LOGIN_COOKIE_KEY, to_cookie_value(login_id.series_id(), login_id.token()));
            request.headers_mut().insert(COOKIE, HeaderValue::from_str(&cookie_str).unwrap());
        }
        
        MockManageSession.manage_session(
            MockInnerService,
            request,
        ).await
    }

    #[tokio::test]
    async fn case_1() {
        assert!(test_case(Some(&*CASE_1), Some(&LoginId::new(LoginSeriesId::gen(), LoginToken::gen()))).await.is_ok());
    }

    #[tokio::test]
    async fn case_2() {
        assert!(test_case(None, Some(&*CASE_2)).await.is_ok());
    }

    #[tokio::test]
    async fn case_3() {
        match test_case(None, Some(&*CASE_3)).await.err() {
            Some(ManageSessionError::InvalidSession(_)) => (),
            _ => panic!()
        }
    }

    #[tokio::test]
    async fn case_4() {
        match test_case(Some(&*CASE_4), None).await.err() {
            Some(ManageSessionError::InvalidSession(_)) => (),
            _ => panic!()
        }
    }

    #[tokio::test]
    async fn case_5() {
        match test_case(None, Some(&*CASE_5)).await.err() {
            Some(ManageSessionError::InvalidSession(_)) => (),
            _ => panic!()
        }
    }

    #[tokio::test]
    async fn case_6() {
        match test_case(None, None).await.err() {
            Some(ManageSessionError::NoSession) => (),
            _ => panic!()
        }
    }
}