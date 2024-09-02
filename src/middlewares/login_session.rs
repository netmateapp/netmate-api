use std::{future::Future, pin::{pin, Pin}, str::FromStr, sync::Arc, task::{ready, Context, Poll}};

use cookie::{Cookie, SplitCookies};
use deadpool_redis::Pool;
use http::{header::COOKIE, HeaderMap, Request};
use pin_project::pin_project;
use scylla::Session;
use thiserror::Error;
use tower::{Layer, Service};

use crate::common::session::{dsl::SessionManagementId, interpreter::{LOGIN_COOKIE_KEY, SESSION_MANAGEMENT_COOKIE_KEY}};

pub struct LoginSessionLayer {
    db: Arc<Session>,
    cache: Arc<Pool>,
}

impl LoginSessionLayer {
    pub fn new(db: Arc<Session>, cache: Arc<Pool>) -> Self {
        LoginSessionLayer { db, cache }
    }
}

impl<S: Clone> Layer<S> for LoginSessionLayer {
    type Service = AccountResolver<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AccountResolver {
            inner,
            db: self.db.clone(),
            cache: self.cache.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AccountResolver<S> {
    inner: S,
    db: Arc<Session>,
    cache: Arc<Pool>,
}

impl <S> AccountResolver<S> {
    fn new(inner: S, db: Arc<Session>, cache: Arc<Pool>) -> Self {
        Self { inner, db, cache }
    }
}

impl <S, B> Service<Request<B>> for AccountResolver<S>
where
    S: Service<Request<B>> + Clone,
    B: Clone,
    S::Error: Into<anyhow::Error>
{
    type Response = S::Response;
    type Error = SessionError;
    type Future = Machine<B, S>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
        /*match self.inner.poll_ready(cx) {
            Poll::Ready(v) => match v {
                Ok(m) => Poll::Ready(Ok(())),
                Err(e) => Poll::Ready(Err(SessionError(e.into())))
            },
            Poll::Pending => Poll::Pending
        } */
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        // __Host-id1=(11)<24>; (2)__Host-id2=(11)<24>$(1)<24> = 97bytes;
        // https://developer.mozilla.org/ja/docs/Web/HTTP/Headers/Cookie
        /*const MAX_COOKIE_BYTES: usize = 100;

        let cookies = headers_to_optional_cookies(req.headers());

        match cookies {
            Some(cookies) => {
                let (session_management_cookie, login_cookie) = get_session_management_cookie_and_login_cookie(cookies);

                if let Some(session_management_id) = session_management_cookie {
                    let session_management_id = SessionManagementId::from_str(session_management_id.value());
                    
                    let session_management_id = session_management_id.unwrap();
                    //let res = ready!(pin!(check_session_existence(&session_management_id)).poll(cx));
                }
                
                if let Some(logion_id) = login_cookie {

                // insert series, timestamp ttl 400days;
                // ↑現状最も長い日数。ブラウザの制限の厳格化で更に短くなる可能性がある。いずれにせよ削除の*自動化*が重要。
                // per 30m: select series, timestamp from...; now - timestamp >= 閾値月数; update ttl 400days;
                // ↑この場合、最大で400日、最短で400 - (閾値月数 * 30)日で永続セッションが消える可能性がある
                // ex. update直後からログインしなくなった場合は400日後にセッションが無効化、
                //     閾値月数経過直前からログインしなくなった場合は、400 - (閾値月数 * 30)日後にセッションが無効化
                
                let v = async {};
                let mut fut = v.into_future();
                } else {
                    // Poll::Ready(Ok(()))
                }
            },
            None => (), //Poll::Ready(Ok(()))
        }
        self.inner.call(req)*/

        Machine {
            inner: self.inner.clone(),
            req
        }
    }
}

#[derive(Debug, Error)]
#[error("")]
pub struct SessionError(#[source] anyhow::Error);

#[pin_project]
pub struct Machine<B, S: Service<Request<B>>> {
    inner: S,
    req: Request<B>,
}

impl<B, S> Future for Machine<B, S>
where
    B: Clone,
    S: Service<Request<B>> + Clone,
    S::Error: Into<anyhow::Error>,
{
    type Output = Result<S::Response, SessionError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let cookies = headers_to_optional_cookies(&self.req.headers());
        let (session_management_cookie, login_cookie) = get_session_management_cookie_and_login_cookie(cookies.unwrap());
        
        if let Some(session_management_id) = session_management_cookie {
            let id = SessionManagementId::from_str(session_management_id.value());

            let id = id.unwrap();
            let res = ready!(pin!(check_session_existence(&id)).poll(cx));
            let res2 = ready!(pin!(check_session_existence2(&id)).poll(cx));
        }
        //loop {
            /*match m { //(*self).state {
                StateMachine::Start(state) => {//(state) => {
                    let cookies = headers_to_optional_cookies(&state.headers);
                    if cookies.is_none() {
                        let sm = self.get_mut();
                        sm.state = StateMachine::End; // with Error ?
                        ControlFlow::Continue(());
                    }

                    let (session_management_cookie, login_cookie) = get_session_management_cookie_and_login_cookie(cookies.unwrap());

                    if let Some(session_management_id) = session_management_cookie {
                        let id = SessionManagementId::from_str(session_management_id.value());
                        if let Err(e) = id {
                            let sm = self.get_mut();
                            sm.state = StateMachine::End; // eを渡す
                            ControlFlow::Continue(());
                        }

                        let id = id.unwrap();
                        let fut = ready!(pin!(check_session_existence(&id)).poll(cx));
                        let state = WaitingOnSessionExistenceState { id };
                        let sm = self.get_mut();
                        let new_state = StateMachine::WaitingOnSessionExistence(state);
                        sm.state = new_state;//(state);
                    }
                    
                    if let Some(logion_id) = login_cookie {

                    // insert series, timestamp ttl 400days;
                    // ↑現状最も長い日数。ブラウザの制限の厳格化で更に短くなる可能性がある。いずれにせよ削除の*自動化*が重要。
                    // per 30m: select series, timestamp from...; now - timestamp >= 閾値月数; update ttl 400days;
                    // ↑この場合、最大で400日、最短で400 - (閾値月数 * 30)日で永続セッションが消える可能性がある
                    // ex. update直後からログインしなくなった場合は400日後にセッションが無効化、
                    //     閾値月数経過直前からログインしなくなった場合は、400 - (閾値月数 * 30)日後にセッションが無効化
                    } else {
                        (*self).state = StateMachine::End
                    }
                },
                StateMachine::WaitingOnSessionExistence(state) => {//(state) => {

                },
                StateMachine::End => {
                    
                }
            }*/
        //}

        let req = self.req.clone();
        let call = self.project().inner.call(req);
        let res = ready!(pin!(call).poll(cx));
        match res {
            Ok(v) => Poll::Ready(Ok(v)),
            Err(e) => Poll::Ready(Err(SessionError(e.into())))
        }
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








/*
use tower::Service;
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;
use std::io::Result as IoResult;
use futures::future::{ready, Ready};
use tokio::runtime::Runtime;

// リクエストとレスポンスの型を仮定します
#[derive(Clone, Debug)]
struct Request;
#[derive(Clone, Debug)]
struct Response;
#[derive(Clone, Debug)]
struct MyError;

impl From<std::io::Error> for MyError {
    fn from(_: std::io::Error) -> Self {
        MyError
    }
}

// 非同期I/Oをシミュレートする関数
fn perform_async_io() -> impl Future<Output = IoResult<()>> {
    ready(Ok(())) // 実際には非同期I/O操作を行うべき
}

// `MyService`を定義し、`tower::Service`を実装
struct MyService;

impl Service<Request> for MyService {
    type Response = Response;
    type Error = MyError;
    type Future = MyFuture;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(())) // 常に準備完了とするシンプルな例
    }

    fn call(&mut self, req: Request) -> Self::Future {
        MyFuture::new(req) // 手動で実装するFutureを返す
    }
}

// ステートマシンを表す列挙型
enum MyFutureState {
    Start(Request),
    Step1,  // 非同期処理のステップ1
    Step2,  // 非同期処理のステップ2
    Done(Result<Response, MyError>),
}

// 非同期処理を管理する`Future`型
struct MyFuture {
    state: MyFutureState,
}

impl MyFuture {
    fn new(req: Request) -> Self {
        MyFuture {
            state: MyFutureState::Start(req),
        }
    }
}

// `Future`トレイトを実装
impl Future for MyFuture {
    type Output = Result<Response, MyError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.state {
                MyFutureState::Start(ref _req) => {
                    // 非同期処理の初期化やステップ1を開始
                    self.state = MyFutureState::Step1;
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                MyFutureState::Step1 => {
                    // 非同期I/O操作を実行
                    match Pin::new(&mut perform_async_io()).poll(cx) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(Ok(())) => {
                            self.state = MyFutureState::Step2;
                            cx.waker().wake_by_ref();
                            return Poll::Pending;
                        }
                        Poll::Ready(Err(e)) => {
                            self.state = MyFutureState::Done(Err(MyError::from(e)));
                        }
                    }
                }
                MyFutureState::Step2 => {
                    // ステップ2の処理が完了した後、結果を生成
                    let response = Response {};
                    self.state = MyFutureState::Done(Ok(response));
                }
                MyFutureState::Done(ref result) => {
                    // 完了したので結果を返す
                    return Poll::Ready(result.clone());
                }
            }
        }
    }
}

fn main() {
    let rt = Runtime::new().unwrap();
    let mut service = MyService;

    let request = Request; // リクエストを作成

    rt.block_on(async {
        // サービスが準備完了か確認
        if let Poll::Ready(Ok(())) = service.poll_ready(&mut Context::from_waker(futures::task::noop_waker_ref())) {
            // リクエストをサービスに送信し、レスポンスを待つ
            let future = service.call(request);
            match future.await {
                Ok(response) => {
                    // レスポンスを処理
                    println!("Response received");
                }
                Err(e) => {
                    // エラー処理
                    eprintln!("Service error: {:?}", e);
                }
            }
        }
    });
}



*/