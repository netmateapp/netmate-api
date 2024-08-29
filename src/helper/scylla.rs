use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, transport::errors::QueryError, Session};

pub async fn prepare<T: From<QueryError>>(session: &Arc<Session>, query: &str) -> Result<Arc<PreparedStatement>, T> {
    match session.prepare(query).await {
        Ok(statement) => Ok(Arc::new(statement)),
        Err(e) => Err(T::from(e))
    }
}