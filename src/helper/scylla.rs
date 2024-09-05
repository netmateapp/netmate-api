use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, transport::errors::QueryError, Session};

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