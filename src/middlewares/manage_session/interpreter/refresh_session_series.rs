use std::sync::Arc;

use scylla::{frame::value::CqlTimestamp, prepared_statement::PreparedStatement, FromRow, Session};

use crate::{common::{fallible::Fallible, id::AccountId, session::value::SessionSeries, unixtime::UnixtimeMillis}, helper::scylla::{Statement, TypedStatement}, middlewares::manage_session::dsl::{manage_session::RefreshPairExpirationSeconds, refresh_session_series::{LastSessionSeriesRefreshedAt, RefreshSessionSeries, RefreshSessionSeriesError, SessionSeriesRefreshThereshold}}};

use super::ManageSessionImpl;

const REFRESH_SESSION_SERIES_THERESHOLD: SessionSeriesRefreshThereshold = SessionSeriesRefreshThereshold::days(30);

impl RefreshSessionSeries for ManageSessionImpl {
    async fn fetch_last_session_series_refreshed_at(&self, session_series: &SessionSeries, session_account_id: &AccountId) -> Fallible<LastSessionSeriesRefreshedAt, RefreshSessionSeriesError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> RefreshSessionSeriesError {
            RefreshSessionSeriesError::FetchLastSessionSeriesRefreshedAtFailed(e.into())
        }

        self.db
            .execute_unpaged(&self.select_last_session_series_refreshed_at, (session_account_id.value().value(), session_series.value().value()))
            .await
            .map_err(handle_error)?
            .first_row_typed::<(CqlTimestamp, )>()
            .map_err(handle_error)
            .map(|(refreshed_at, )| LastSessionSeriesRefreshedAt::new(UnixtimeMillis::from(refreshed_at.0)))
    }

    fn refresh_thereshold() -> &'static SessionSeriesRefreshThereshold {
        &REFRESH_SESSION_SERIES_THERESHOLD
    }

    async fn refresh_session_series(&self, session_series: &SessionSeries, session_account_id: &AccountId, new_expiration: &RefreshPairExpirationSeconds) -> Fallible<(), RefreshSessionSeriesError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> RefreshSessionSeriesError {
            RefreshSessionSeriesError::RefreshSessionSeriesFailed(e.into())
        }

        let values = (
            session_account_id.to_string(),
            session_series.to_string(),
            i64::from(UnixtimeMillis::now()),
            i32::from(new_expiration.clone())
        );

        self.db
            .execute_unpaged(&self.update_session_series_ttl, values)
            .await
            .map(|_| ())
            .map_err(handle_error)
    }
}

const SELECT_LAST_API_KEY_REFRESHED_AT: Statement<SelectLastSessionSeriesRefreshedAt> = Statement::of("SELECT refreshed_at FROM session_series WHERE account_id = ? AND series = ? LIMIT 1");
struct SelectLastSessionSeriesRefreshedAt(Arc<PreparedStatement>);

impl<'a> TypedStatement<(&'a AccountId, &'a SessionSeries), (LastSessionSeriesRefreshedAt, )> for SelectLastSessionSeriesRefreshedAt {
    type Result<U> = U where U: FromRow;

    async fn query(&self, db: &Arc<Session>, values: (&'a AccountId, &'a SessionSeries)) -> anyhow::Result<Self::Result<(LastSessionSeriesRefreshedAt, )>> {
        db.execute_unpaged(&self.0, values)
            .await
            .map_err(anyhow::Error::from)?
            .first_row_typed()
            .map_err(anyhow::Error::from)
    }
}