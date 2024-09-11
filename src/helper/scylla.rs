use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, serialize::row::SerializeRow, transport::errors::QueryError, FromRow, Session};

use super::error::InitError;

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

#[macro_export]
macro_rules! cql {
    ($query:expr) => {
        $query
    };
}

fn a() {
    let _s = cql!("SELECT * FROM table");
}

pub(crate) trait TypedStatement<V, R> {
    type Output: FromRow;
    type Error;

    fn serialize_values(&self, values: V) -> impl SerializeRow;

    fn deserialize_values(values: Self::Output) -> Result<R, Self::Error>; 
}
