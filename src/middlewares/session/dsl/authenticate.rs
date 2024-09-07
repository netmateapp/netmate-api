use std::convert::Infallible;

use http::{Request, Response};
use thiserror::Error;
use tower::Service;

use crate::common::{id::AccountId, session::value::SessionId};

use super::{dsl::{can_set_cookie_in_response_header, insert_account_id_into_request_extensions}, set_cookie::set_new_session_id_into_response_header};

pub(crate) trait AuthenticateUser {
    async fn authenticate<S, B>(&self, inner: &mut S, mut req: Request<B>, session_id: &SessionId, account_id: AccountId) -> S::Response
    where
        S: Service<Request<B>, Error = Infallible, Response = Response<B>>,
    {
        insert_account_id_into_request_extensions(&mut req, account_id);

        // `Error`は`Infallible`で起こり得ないので`unwrap()`で問題ない
        let mut response = inner.call(req).await.unwrap();

        // パスワード変更やログアウトによるSet-Cookieヘッダが無い場合のみセッションを延長
        if can_set_cookie_in_response_header(&mut response) {
            Self::reset_session_timeout(&mut response, &session_id);
        }

        response
    }


    fn reset_session_timeout<B>(response: &mut Response<B>, session_id: &SessionId) {
        // 書き直し必要
        // 期限の延長はクッキーの再設定により行われるため、実態はセッション識別子の再設定関数である
        set_new_session_id_into_response_header(response.headers_mut(), session_id);
    }
}

#[derive(Debug, Error)]
pub enum AuthenticateUserError {
    #[error("セッションIDの解決に失敗しました")]
    ResolveSessionIdFailed,
    #[error("無効なセッションIDです")]
    InvalidSessionId,
}