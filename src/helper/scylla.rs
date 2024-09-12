use std::{marker::PhantomData, sync::Arc};

use scylla::{prepared_statement::PreparedStatement, serialize::row::SerializeRow, transport::errors::QueryError, FromRow, Session};

use super::error::InitError;

#[macro_export]
macro_rules! cql {
    ($query:expr) => {
        $query
    };
}

pub async fn prep<T: From<QueryError>>(session: &Arc<Session>, query: &str) -> Result<Arc<PreparedStatement>, T> {
    match session.prepare(query).await {
        Ok(statement) => Ok(Arc::new(statement)),
        Err(e) => Err(T::from(e))
    }
}

impl<T> From<QueryError> for InitError<T> {
    fn from(value: QueryError) -> Self {
        Self::new(value.into())
    }
}

pub struct Statement<T>(&'static str, PhantomData<T>);

impl<T> Statement<T> {
    pub const fn of(statement: &'static str) -> Self {
        Self(statement, PhantomData)
    }

    #[cfg(debug_assertions)]
    pub fn value(&self) -> &str {
        self.0
    }
}

pub(crate) async fn prepare<I, O, T, C>(session: &Arc<Session>, constructor: C, statement: Statement<T>) -> Result<T, QueryError>
where
    I: SerializeRow,
    O: FromRow,
    T: TypedStatement<I, O>,
    C: FnOnce(Arc<PreparedStatement>) -> T
{
    match session.prepare(statement.0).await {
        Ok(statement) => Ok(constructor(Arc::new(statement))),
        Err(e) => Err(e)
    }
}

pub(crate) trait TypedStatement<I, O>
where
    Self: Sized,
    I: SerializeRow,
    O: FromRow,
{
    async fn execute(&self, db: &Arc<Session>, values: I) -> anyhow::Result<O>;
}