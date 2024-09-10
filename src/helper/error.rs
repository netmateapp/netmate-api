use std::marker::PhantomData;

use thiserror::Error;

// エンドポイントで使用する
#[derive(Debug, Error)]
#[error("初期化に失敗しました")]
pub struct InitError<T>(#[source] anyhow::Error, PhantomData<fn() -> T>);

impl<T> InitError<T> {
    pub fn new(error: anyhow::Error) -> Self {
        InitError(error, PhantomData)
    }
}
