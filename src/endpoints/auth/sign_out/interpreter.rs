use std::sync::Arc;

use redis::cmd;
use scylla::{prepared_statement::PreparedStatement, FromRow, Session};

use crate::{common::{fallible::Fallible, id::account_id::AccountId, session::session_series::SessionSeries}, helper::{error::InitError, redis::{Connection, Pool, TypedCommand, DEL_COMMAND}, scylla::{Statement, TypedStatement, Unit}}, middlewares::session::RefreshPairKey};

use super::dsl::{SignOut, SignOutError};

pub struct SignOutImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    delete_session_series: Arc<DeleteSessionSeries>,
}

impl SignOut for SignOutImpl {
    async fn sign_out(&self, account_id: AccountId, session_series: &SessionSeries) -> Fallible<(), SignOutError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> SignOutError {
            SignOutError::SignOutFailed(e.into())
        }

        let refresh_pair_key = RefreshPairKey(session_series);
        DeleteRefreshPairCommand.run(&self.cache, refresh_pair_key)
            .await
            .map_err(handle_error)?;

        self.delete_session_series
            .execute(&self.db, (account_id, session_series))
            .await
            .map_err(handle_error)
    }
}

impl SignOutImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<Self>> {
        let delete_session_series = DELETE_SESSION_SERIES.prepared(&db, DeleteSessionSeries)
            .await
            .map_err(|e| InitError::new(e.into()))?;

        Ok(Self { db, cache, delete_session_series })
    }
}

struct DeleteRefreshPairCommand;

impl<'a> TypedCommand<RefreshPairKey<'a>, ()> for DeleteRefreshPairCommand {
    async fn execute(&self, mut conn: Connection<'_>, refresh_pair_key: RefreshPairKey<'a>) -> anyhow::Result<()> {
        cmd(DEL_COMMAND)
            .arg(refresh_pair_key)
            .exec_async(&mut *conn)
            .await
            .map_err(anyhow::Error::from)
    }
}

const DELETE_SESSION_SERIES: Statement<DeleteSessionSeries>
    = Statement::of("DELETE FROM session_series WHERE account_id = ? AND series = ?");

struct DeleteSessionSeries(PreparedStatement);

impl<'a> TypedStatement<(AccountId, &'a SessionSeries), Unit> for DeleteSessionSeries {
    type Result<U> = U where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (AccountId, &'a SessionSeries)) -> anyhow::Result<Unit> {
        session.execute_unpaged(&self.0, values)
            .await
            .map(|_| Unit)
            .map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::helper::scylla::check_cql_statement_type;

    use super::DELETE_SESSION_SERIES;

    #[test]
    fn check_delete_session_series_type() {
        check_cql_statement_type(DELETE_SESSION_SERIES);
    }
}