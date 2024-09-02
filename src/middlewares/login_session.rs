use std::{future::Future, pin::{pin, Pin}, str::FromStr, sync::Arc, task::{ready, Context, Poll}};

use bb8_redis::{bb8::Pool, redis::cmd, RedisConnectionManager};
use cookie::{Cookie, SplitCookies};
use http::{header::COOKIE, HeaderMap, Request};
use pin_project::pin_project;
use scylla::Session;
use tower::{Layer, Service};

use crate::common::session::{dsl::SessionManagementId, interpreter::{LOGIN_COOKIE_KEY, SESSION_MANAGEMENT_COOKIE_KEY}};

type Connection = Pool<RedisConnectionManager>;

#[derive(Clone)]
pub struct LoginSessionLayer {
    db: Arc<Session>,
    cache: Arc<Connection>,
}

impl LoginSessionLayer {
    pub fn new(db: Arc<Session>, cache: Arc<Connection>) -> Self {
        LoginSessionLayer { db, cache }
    }
}

impl<S> Layer<S> for LoginSessionLayer {
    type Service = LoginSessionService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoginSessionService {
            inner,
            db: self.db.clone(),
            cache: self.cache.clone(),
        }
    }
}

#[derive(Clone)]
pub struct LoginSessionService<S> {
    inner: S,
    db: Arc<Session>,
    cache: Arc<Connection>,
}

impl <S, B> Service<Request<B>> for LoginSessionService<S>
where
    S: Service<Request<B>>,
    S::Error: Into<anyhow::Error>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = SessionFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let cookies = headers_to_optional_cookies(req.headers());
        let res = get_session_management_cookie_and_login_cookie(cookies.unwrap());
        let cookies = cookies_to_values(res);

        let response_future = self.inner.call(req);

        SessionFuture {
            response_future,
            cookies,
            db: self.db.clone(),
            cache: self.cache.clone()
        }
    }
}

#[pin_project]
pub struct SessionFuture<F> {
    #[pin]
    response_future: F,
    cookies: (Option<String>, Option<String>),
    db: Arc<Session>,
    cache: Arc<Connection>,
}

impl<F, R, E> Future for SessionFuture<F>
where
    F: Future<Output = Result<R, E>>,
    E: Into<anyhow::Error>,
{
    type Output = Result<R, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        /*
        処理のパターンは5通り(S: セッション管理識別子, L: ログイン識別子)
        1. S (通常のセッション認証、これが最も多い)
        2. None/Fail(S) -> L (セッションの更新、次に多い)
        3. None/Fail(S) -> Fail(L) (セッション削除後/期限切れ後の場合、まれにある)
        4. Fail(S) -> None(L) (普通はない、クライアント側でユーザーが何らかの操作を行っている可能性がある)
        5. None(S) -> None(L) (UIからは送れないはず、UI外でエンドポイントを叩いている可能性が高い)
         */

        if let Some(session_management_id) = &self.cookies.0 {
            if let Ok(id) = SessionManagementId::from_str(session_management_id) {
                let mut conn = ready!(pin!(self.cache.get()).poll(cx));
                
                match conn {
                    Ok(conn) => {
                        let key = format!("{}:{}", "", id.value());
                        let res = ready!(pin!(cmd("GET").arg(key).query_async(&mut *conn)).poll(cx));
                    },
                    Err(e) => return Poll::Ready(Err(e))
                }
            }
            //let res = ready!(pin!(check_session_existence(&id)).poll(cx));
        }

        if let Some(logion_id) = &self.cookies.1 { //login_cookie {
            // insert series, timestamp ttl 400days;
            // ↑現状最も長い日数。ブラウザの制限の厳格化で更に短くなる可能性がある。いずれにせよ削除の*自動化*が重要。
            // per 30m: select series, timestamp from...; now - timestamp >= 閾値月数; update ttl 400days;
            // ↑この場合、最大で400日、最短で400 - (閾値月数 * 30)日で永続セッションが消える可能性がある
            // ex. update直後からログインしなくなった場合は400日後にセッションが無効化、
            //     閾値月数経過直前からログインしなくなった場合は、400 - (閾値月数 * 30)日後にセッションが無効化
        }

        let future = self.project().response_future;
        let res = ready!(future.poll(cx));
        Poll::Ready(res)
    }
}

fn headers_to_optional_cookies(headers: &HeaderMap) -> Option<SplitCookies<'_>> {
    // __Host-id1=(11)<24>; (2)__Host-id2=(11)<24>$(1)<24> = 97bytes;
    // https://developer.mozilla.org/ja/docs/Web/HTTP/Headers/Cookie
    const MAX_COOKIE_BYTES: usize = 100;

    headers.get(COOKIE)
        .and_then(|cookie_header| cookie_header.to_str().ok())
        .filter(|cookie_str| cookie_str.len() <= MAX_COOKIE_BYTES)
        .map(|cookie_str| Cookie::split_parse(cookie_str))
}

fn get_session_management_cookie_and_login_cookie(cookies: SplitCookies<'_>) -> (Option<Cookie<'_>>, Option<Cookie<'_>>) {
    let mut session_management_cookie = None;
    let mut login_cookie = None;

    // セッション管理クッキーとログインクッキーがあれば取得する
    for cookie in cookies {
        match cookie {
            Ok(cookie) => match cookie.name() {
                SESSION_MANAGEMENT_COOKIE_KEY => session_management_cookie = Some(cookie),
                LOGIN_COOKIE_KEY => login_cookie = Some(cookie),
                _ => ()
            },
            _ => ()
        }
    }

    (session_management_cookie, login_cookie)
}

fn cookies_to_values(cookies: (Option<Cookie<'_>>, Option<Cookie<'_>>)) -> (Option<String>, Option<String>) {
    (cookies.0.map(|c| c.value().to_string()), cookies.1.map(|c| c.value().to_string()))
}

async fn check_session_existence(session_management_id: &SessionManagementId) {

}

async fn check_session_existence2(session_management_id: &SessionManagementId) -> Result<bool, ()> {
    Ok(true)
}

/*
    fn call(&mut self, mut req: Request<B>) -> Self::Future {
         Box::pin(async move {
            if let Some(cookie_header) = req.headers().get(COOKIE) {
                if let Ok(cookie_str) = cookie_header.to_str() {
                    if let Ok(cookie) = Cookie::parse(cookie_str) {
                        if let Some(session_id) = cookie.get("session_id") {
                            req.extensions_mut().insert(SessionId(session_id.to_string()));
                        }
                    }
                }
            }
            self.inner.call(req).await
        })
    }
} */