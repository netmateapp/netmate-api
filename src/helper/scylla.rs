use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, serialize::row::SerializeRow, transport::errors::QueryError, FromRow, Session};

use super::error::InitError;

#[macro_export]
macro_rules! cql {
    ($query:expr) => {
        $query
    };
}

pub async fn prepare<T: From<QueryError>>(session: &Arc<Session>, query: &str) -> Result<Arc<PreparedStatement>, T> {
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

pub(crate) trait TypedStatement<I, O>
where
    I: SerializeRow,
    O: FromRow,
{
    async fn execute(&self, db: &Arc<Session>, values: I) -> anyhow::Result<O>;
}