use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, FromRow, Session};

use crate::{common::{fallible::Fallible, id::account_id::AccountId, session::{refresh_pair_expiration::RefreshPairExpirationSeconds, session_series::SessionSeries}, unixtime::UnixtimeMillis}, helper::scylla::{Statement, TypedStatement, Unit}, middlewares::manage_session::dsl::refresh_session_series::{LastSessionSeriesRefreshedAt, RefreshSessionSeries, RefreshSessionSeriesError, SessionSeriesRefreshThereshold}};

use super::ManageSessionImpl;

const REFRESH_SESSION_SERIES_THERESHOLD: SessionSeriesRefreshThereshold = SessionSeriesRefreshThereshold::days(30);

impl RefreshSessionSeries for ManageSessionImpl {
    async fn fetch_last_session_series_refreshed_at(&self, session_series: &SessionSeries, session_account_id: AccountId) -> Fallible<LastSessionSeriesRefreshedAt, RefreshSessionSeriesError> {
        self.select_last_session_series_refreshed_at
            .query(&self.db, (session_account_id, session_series))
            .await
            .map(|(refreshed_at, )| refreshed_at)
            .map_err(RefreshSessionSeriesError::FetchLastSessionSeriesRefreshedAtFailed)
    }

    fn refresh_thereshold() -> &'static SessionSeriesRefreshThereshold {
        &REFRESH_SESSION_SERIES_THERESHOLD
    }

    async fn refresh_session_series(&self, session_series: &SessionSeries, session_account_id: AccountId, new_expiration: RefreshPairExpirationSeconds) -> Fallible<(), RefreshSessionSeriesError> {
        let values = (session_account_id, session_series, UnixtimeMillis::now(), new_expiration);

        self.update_session_series_ttl
            .execute(&self.db, values)
            .await
            .map_err(RefreshSessionSeriesError::RefreshSessionSeriesFailed)
    }
}

// 以下、型付きCQL文の定義
pub const SELECT_LAST_API_KEY_REFRESHED_AT: Statement<SelectLastSessionSeriesRefreshedAt>
    = Statement::of("SELECT refreshed_at FROM session_series WHERE account_id = ? AND series = ? LIMIT 1");

#[derive(Debug)]
pub struct SelectLastSessionSeriesRefreshedAt(pub PreparedStatement);

impl<'a> TypedStatement<(AccountId, &'a SessionSeries), (LastSessionSeriesRefreshedAt, )> for SelectLastSessionSeriesRefreshedAt {
    type Result<U> = U where U: FromRow;

    async fn query(&self, db: &Arc<Session>, values: (AccountId, &'a SessionSeries)) -> anyhow::Result<Self::Result<(LastSessionSeriesRefreshedAt, )>> {
        db.execute_unpaged(&self.0, values)
            .await
            .map_err(anyhow::Error::from)?
            .first_row_typed()
            .map_err(anyhow::Error::from)
    }
}

pub const UPDATE_SESSION_SERIES_TTL: Statement<UpdateSessionSeriesTtl>
    = Statement::of("UPDATE session_series SET refreshed_at = ? WHERE account_id = ? AND series = ? USING TTL ?");

#[derive(Debug)]
pub struct UpdateSessionSeriesTtl(pub PreparedStatement);

impl<'a> TypedStatement<(AccountId, &'a SessionSeries, UnixtimeMillis, RefreshPairExpirationSeconds), Unit> for UpdateSessionSeriesTtl {
    type Result<U> = U where U: FromRow;

    async fn query(&self, db: &Arc<Session>, values: (AccountId, &'a SessionSeries, UnixtimeMillis, RefreshPairExpirationSeconds)) -> anyhow::Result<Self::Result<Unit>> {
        db.execute_unpaged(&self.0, values)
            .await
            .map_err(anyhow::Error::from)?
            .first_row_typed()
            .map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::helper::scylla::{check_cql_query_type, check_cql_statement_type};

    use super::{SELECT_LAST_API_KEY_REFRESHED_AT, UPDATE_SESSION_SERIES_TTL};

    #[test]
    fn check_select_last_session_series_refreshed_at_type() {
        check_cql_query_type(SELECT_LAST_API_KEY_REFRESHED_AT);
    }

    #[test]
    fn check_update_session_series_ttl_type() {
        check_cql_statement_type(UPDATE_SESSION_SERIES_TTL);
    }
}