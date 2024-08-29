use std::marker::PhantomData;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("初期化に失敗しました")]
pub struct InitError<T>(#[source] anyhow::Error, PhantomData<fn() -> T>);

impl<T> InitError<T> {
    pub fn new(error: anyhow::Error, _phantom: PhantomData<fn() -> T>) -> Self {
        InitError(error, _phantom)
    }
}