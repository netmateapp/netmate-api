use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::CqlValue};
use thiserror::Error;

use crate::common::{fallible::Fallible, id::AccountId, session::{refresh_pair_expiration::RefreshPairExpiration, session_series::SessionSeries}, unixtime::UnixtimeMillis};

pub(crate) trait RefreshSessionSeries {
    // 指定されたセッション系列が存在している前提で実行される
    async fn try_refresh_session_series(&self, session_series: &SessionSeries, session_account_id: AccountId, new_expiration: RefreshPairExpiration) -> Fallible<(), RefreshSessionSeriesError> {
        let last_refreshed_at = self.fetch_last_session_series_refreshed_at(session_series, session_account_id).await?;
        if Self::should_refresh_session_series(&last_refreshed_at) {
            self.refresh_session_series(session_series, session_account_id, new_expiration).await
        } else {
            Ok(())
        }
    }

    async fn fetch_last_session_series_refreshed_at(&self, session_series: &SessionSeries, session_account_id: AccountId) -> Fallible<LastSessionSeriesRefreshedAt, RefreshSessionSeriesError>;

    fn should_refresh_session_series(last_refreshed_at: &LastSessionSeriesRefreshedAt) -> bool {
        let now = UnixtimeMillis::now();
        let last_refreshed_at = last_refreshed_at.as_unixtime_millis();
        now.value() - last_refreshed_at.value() >= Self::refresh_thereshold().as_millis()
    }

    fn refresh_thereshold() -> &'static SessionSeriesRefreshThereshold;

    async fn refresh_session_series(&self, session_series: &SessionSeries, session_account_id: AccountId, new_expiration: RefreshPairExpiration) -> Fallible<(), RefreshSessionSeriesError>;
}

#[derive(Debug, Error)]
pub enum RefreshSessionSeriesError {
    #[error("セッションシリーズの最終更新時刻の取得に失敗しました")]
    FetchLastSessionSeriesRefreshedAtFailed(#[source] anyhow::Error),
    #[error("セッションシリーズの更新に失敗しました")]
    RefreshSessionSeriesFailed(#[source] anyhow::Error),
}

pub struct LastSessionSeriesRefreshedAt(UnixtimeMillis);

impl LastSessionSeriesRefreshedAt {
    pub const fn new(time: UnixtimeMillis) -> Self {
        Self(time)
    }

    pub fn as_unixtime_millis(&self) -> &UnixtimeMillis {
        &self.0
    }
}

impl FromCqlVal<Option<CqlValue>> for LastSessionSeriesRefreshedAt {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        UnixtimeMillis::from_cql(cql_val).map(Self)
    }
}

pub struct SessionSeriesRefreshThereshold(u64);

impl SessionSeriesRefreshThereshold {
    pub const fn days(days: u64) -> Self {
        Self(days)
    }

    pub fn as_millis(&self) -> u64 {
        self.0 * 24 * 60 * 60 * 1000
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use crate::common::{fallible::Fallible, id::{uuid7::Uuid7, AccountId}, session::{refresh_pair_expiration::RefreshPairExpiration, session_series::SessionSeries}, unixtime::UnixtimeMillis};

    use super::{LastSessionSeriesRefreshedAt, RefreshSessionSeries, RefreshSessionSeriesError, SessionSeriesRefreshThereshold};

    const SESSION_SERIES_TO_BE_REFRESHED: LazyLock<SessionSeries> = LazyLock::new(|| SessionSeries::gen());
    const REFRESH_THERESHOLD: SessionSeriesRefreshThereshold = SessionSeriesRefreshThereshold::days(1);

    struct MockRefreshSessionSeries;

    impl RefreshSessionSeries for MockRefreshSessionSeries {
        async fn fetch_last_session_series_refreshed_at(&self, session_series: &SessionSeries, _session_account_id: AccountId) -> Fallible<LastSessionSeriesRefreshedAt, RefreshSessionSeriesError> {
            if session_series == &*SESSION_SERIES_TO_BE_REFRESHED {
                let last_refreshed_at = UnixtimeMillis::now().value() - REFRESH_THERESHOLD.as_millis();
                Ok(LastSessionSeriesRefreshedAt::new(UnixtimeMillis::new(last_refreshed_at)))
            } else {
                Ok(LastSessionSeriesRefreshedAt::new(UnixtimeMillis::now()))
            }
        }

        fn refresh_thereshold() -> &'static SessionSeriesRefreshThereshold {
            &REFRESH_THERESHOLD
        }

        async fn refresh_session_series(&self, _session_series: &SessionSeries, _session_account_id: AccountId, _new_expiration: RefreshPairExpiration) -> Fallible<(), RefreshSessionSeriesError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn session_series_to_be_refreshed() {
        let result = MockRefreshSessionSeries.try_refresh_session_series(
            &*SESSION_SERIES_TO_BE_REFRESHED,
            AccountId::of(Uuid7::now()),
            RefreshPairExpiration::secs(1),
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn session_series_to_not_be_refreshed() {
        let result = MockRefreshSessionSeries.try_refresh_session_series(
            &SessionSeries::gen(),
            AccountId::of(Uuid7::now()),
            RefreshPairExpiration::secs(1),
        ).await;
        assert!(result.is_ok());
    }
}