use std::sync::Arc;

use redis::cmd;
use scylla::{prepared_statement::PreparedStatement, FromRow, Session};

use crate::{common::{fallible::Fallible, id::account_id::AccountId, session::{refresh_pair_expiration::RefreshPairExpirationSeconds, refresh_token::RefreshToken, session_series::SessionSeries}, unixtime::UnixtimeMillis}, helper::{redis::{Connection, TypedCommand, EX_OPTION, NX_OPTION, SET_COMMAND}, scylla::{Statement, TypedStatement, Unit}}, middlewares::{session::{RefreshPairKey, RefreshPairValue}, start_session::dsl::assign_refresh_pair::{AssignRefreshPair, AssignRefreshPairError}}};

use super::StartSessionImpl;

impl AssignRefreshPair for StartSessionImpl {
    async fn try_assign_refresh_pair_with_expiration_if_unused(&self, session_series: &SessionSeries, refresh_token: &RefreshToken, session_account_id: AccountId, expiration: RefreshPairExpirationSeconds) -> Fallible<(), AssignRefreshPairError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> AssignRefreshPairError {
            AssignRefreshPairError::AssignRefreshPairFailed(e.into())
        }
        
        let key = RefreshPairKey(session_series);
        let value = RefreshPairValue(refresh_token, session_account_id);

        SetRefreshPairCommand.run(&self.cache, (key, value, expiration))
            .await
            .map_err(handle_error)?
            .map_or_else(|| Err(AssignRefreshPairError::SessionSeriesAlreadyUsed), |_| Ok(()))?;

        self.insert_session_series
            .execute(&self.db, (session_account_id, session_series, UnixtimeMillis::now()))
            .await
            .map(|_| ())
            .map_err(handle_error)
    }
}

struct SetRefreshPairCommand;

impl<'a, 'b> TypedCommand<(RefreshPairKey<'a>, RefreshPairValue<'b>, RefreshPairExpirationSeconds), Option<()>> for SetRefreshPairCommand {
    async fn execute(&self, mut conn: Connection<'_>, (key, value, expiration): (RefreshPairKey<'a>, RefreshPairValue<'b>, RefreshPairExpirationSeconds)) -> anyhow::Result<Option<()>> {
        cmd(SET_COMMAND)
            .arg(key)
            .arg(value)
            .arg(EX_OPTION)
            .arg(expiration)
            .arg(NX_OPTION)
            .query_async::<Option<()>>(&mut *conn)
            .await
            .map_err(Into::into)
    }
}

pub const INSERT_SESSION_SERIES: Statement<InsertSessionSeries>
    = Statement::of("INSERT INTO session_series (account_id, series, refreshed_at) VALUES (?, ?, ?) USING TTL 34560000");

#[derive(Debug)]
pub struct InsertSessionSeries(pub PreparedStatement);

impl<'a> TypedStatement<(AccountId, &'a SessionSeries, UnixtimeMillis), Unit> for InsertSessionSeries {
    type Result<U> = U where U: FromRow;

    async fn query(&self, db: &Arc<Session>, values: (AccountId, &'a SessionSeries, UnixtimeMillis)) -> anyhow::Result<Self::Result<Unit>> {
        db.execute_unpaged(&self.0, values)
            .await
            .map(|_| Unit)
            .map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::helper::scylla::check_cql_statement_type;

    use super::INSERT_SESSION_SERIES;

    #[test]
    fn check_insert_session_series_type() {
        check_cql_statement_type(INSERT_SESSION_SERIES);
    }
}