use crate::{common::{fallible::Fallible, id::account_id::AccountId, session::{refresh_pair_expiration::RefreshPairExpirationSeconds, session_series::SessionSeries}, unixtime::UnixtimeMillis}, middlewares::manage_session::dsl::refresh_session_series::{LastSessionSeriesRefreshedAt, RefreshSessionSeries, RefreshSessionSeriesError, SessionSeriesRefreshThereshold}};

use super::ManageSessionImpl;

const REFRESH_SESSION_SERIES_THERESHOLD: SessionSeriesRefreshThereshold = SessionSeriesRefreshThereshold::days(30);

impl RefreshSessionSeries for ManageSessionImpl {
    async fn fetch_last_session_series_refreshed_at(&self, session_series: &SessionSeries, session_account_id: AccountId) -> Fallible<LastSessionSeriesRefreshedAt, RefreshSessionSeriesError> {
        self.db
            .execute_unpaged(&self.select_last_session_series_refreshed_at, (session_account_id, session_series))
            .await
            .map_err(|e| RefreshSessionSeriesError::FetchLastSessionSeriesRefreshedAtFailed(e.into()))?
            .first_row_typed::<(LastSessionSeriesRefreshedAt, )>()
            .map(|(refreshed_at, )| refreshed_at)
            .map_err(|e| RefreshSessionSeriesError::FetchLastSessionSeriesRefreshedAtFailed(e.into()))
    }

    fn refresh_thereshold() -> &'static SessionSeriesRefreshThereshold {
        &REFRESH_SESSION_SERIES_THERESHOLD
    }

    async fn refresh_session_series(&self, session_series: &SessionSeries, session_account_id: AccountId, new_expiration: RefreshPairExpirationSeconds) -> Fallible<(), RefreshSessionSeriesError> {
        self.db
            .execute_unpaged(&self.update_session_series_ttl, (UnixtimeMillis::now(), session_account_id, session_series, new_expiration))
            .await
            .map(|_| ())
            .map_err(|e| RefreshSessionSeriesError::RefreshSessionSeriesFailed(e.into()))
    }
}