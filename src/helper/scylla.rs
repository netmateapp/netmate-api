use std::sync::Arc;

use scylla::{cql_to_rust::FromCqlVal, prepared_statement::PreparedStatement, transport::errors::QueryError, QueryResult, Session};

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

pub trait Transactional {
    fn applied<E, H, S>(self, error_handler: H, error_on_unapplied: S) -> Result<(), E>
    where
        H: Fn(anyhow::Error) -> E,
        S: Fn() -> E;
}

impl Transactional for Result<QueryResult, QueryError> {
    fn applied<E, H, S>(self, error_handler: H, error_on_unapplied: S) -> Result<(), E>
    where
        H: Fn(anyhow::Error) -> E,
        S: Fn() -> E
    {
        match self {
            Ok(result) => {
                let (applied_idx, _) = result.get_column_spec("applied")
                .ok_or_else(|| error_handler(anyhow::anyhow!("applied列がありません")))?;
        
                let applied = result.first_row()
                    .map_err(|e| error_handler(e.into()))?
                    .columns[applied_idx]
                    .take();
        
                 bool::from_cql(applied)
                    .map_err(|e| error_handler(e.into()))
                    .and_then(|is_applied| if is_applied {
                        Ok(())
                    } else {
                        Err(error_on_unapplied())
                    })
            },
            Err(e) => Err(error_handler(e.into()))
        }
    }
}