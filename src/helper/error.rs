use std::marker::PhantomData;

use thiserror::Error;

// エンドポイントで使用する
#[derive(Debug, Error)]
#[error("初期化に失敗しました")]
pub struct InitError<T>(#[source] anyhow::Error, PhantomData<fn() -> T>);

impl<T> InitError<T> {
    pub fn new(error: anyhow::Error, _phantom: PhantomData<fn() -> T>) -> Self {
        InitError(error, _phantom)
    }
}

// インタプリタで使用する
pub trait DslErrorMapper<T, E> {
    fn map_dsl_error(self) -> Result<T, E>;
}