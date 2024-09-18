use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, transport::errors::QueryError, Session};

use super::error::InitError;

impl<T> From<QueryError> for InitError<T> {
    fn from(value: QueryError) -> Self {
        Self::new(value.into())
    }
}

pub async fn prepare<T>(session: &Arc<Session>, query: &str) -> Result<Arc<PreparedStatement>, InitError<T>> {
    match session.prepare(query).await {
        Ok(statement) => Ok(Arc::new(statement)),
        Err(e) => Err(e.into())
    }
}