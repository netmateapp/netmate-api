use thiserror::Error;

use crate::common::{fallible::Fallible, id::AccountId, session::value::SessionSeries, unixtime::UnixtimeMillis};

use super::manage_session::RefreshPairExpirationSeconds;

pub(crate) trait RefreshSessionSeries {
    // 指定されたセッション系列が存在している前提で実行される
    async fn try_refresh_session_series(&self, session_series: &SessionSeries, session_account_id: &AccountId, new_expiration: &RefreshPairExpirationSeconds) -> Fallible<(), RefreshSessionSeriesError> {
        let last_refreshed_at = self.fetch_last_session_series_refreshed_at(session_series, session_account_id).await?;
        if Self::should_refresh_session_series(&last_refreshed_at) {
            self.refresh_session_series(session_series, session_account_id, new_expiration).await
        } else {
            Ok(())
        }
    }

    async fn fetch_last_session_series_refreshed_at(&self, session_series: &SessionSeries, session_account_id: &AccountId) -> Fallible<LastSessionSeriesRefreshedTime, RefreshSessionSeriesError>;

    fn should_refresh_session_series(last_refreshed_at: &LastSessionSeriesRefreshedTime) -> bool {
        let now = UnixtimeMillis::now();
        let last_refreshed_at = last_refreshed_at.as_unixtime_millis();
        now.value() - last_refreshed_at.value() >= Self::refresh_thereshold().as_millis()
    }

    fn refresh_thereshold() -> &'static RefreshSessionSeriesThereshold;

    async fn refresh_session_series(&self, session_series: &SessionSeries, session_account_id: &AccountId, new_expiration: &RefreshPairExpirationSeconds) -> Fallible<(), RefreshSessionSeriesError>;
}

#[derive(Debug, Error)]
pub enum RefreshSessionSeriesError {
    #[error("セッションシリーズの最終更新時刻の取得に失敗しました")]
    FetchLastSessionSeriesRefreshedAtFailed(#[source] anyhow::Error),
    #[error("セッションシリーズの更新に失敗しました")]
    RefreshSessionSeriesFailed(#[source] anyhow::Error),
}

pub struct LastSessionSeriesRefreshedTime(UnixtimeMillis);

impl LastSessionSeriesRefreshedTime {
    pub const fn new(time: UnixtimeMillis) -> Self {
        Self(time)
    }

    pub fn as_unixtime_millis(&self) -> &UnixtimeMillis {
        &self.0
    }
}

pub struct RefreshSessionSeriesThereshold(u64);

impl RefreshSessionSeriesThereshold {
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

    use crate::{common::{fallible::Fallible, id::{uuid7::Uuid7, AccountId}, session::value::SessionSeries, unixtime::UnixtimeMillis}, middlewares::session::dsl::manage_session::RefreshPairExpirationSeconds};

    use super::{LastSessionSeriesRefreshedTime, RefreshSessionSeries, RefreshSessionSeriesError, RefreshSessionSeriesThereshold};

    const SESSION_SERIES_TO_BE_REFRESHED: LazyLock<SessionSeries> = LazyLock::new(|| SessionSeries::gen());
    const REFRESH_THERESHOLD: RefreshSessionSeriesThereshold = RefreshSessionSeriesThereshold::days(1);

    struct MockRefreshSessionSeries;

    impl RefreshSessionSeries for MockRefreshSessionSeries {
        async fn fetch_last_session_series_refreshed_at(&self, session_series: &SessionSeries, _session_account_id: &AccountId) -> Fallible<LastSessionSeriesRefreshedTime, RefreshSessionSeriesError> {
            if session_series == &*SESSION_SERIES_TO_BE_REFRESHED {
                let last_refreshed_at = UnixtimeMillis::now().value() - REFRESH_THERESHOLD.as_millis();
                Ok(LastSessionSeriesRefreshedTime::new(UnixtimeMillis::new(last_refreshed_at)))
            } else {
                Ok(LastSessionSeriesRefreshedTime::new(UnixtimeMillis::now()))
            }
        }

        fn refresh_thereshold() -> &'static RefreshSessionSeriesThereshold {
            &REFRESH_THERESHOLD
        }

        async fn refresh_session_series(&self, _session_series: &SessionSeries, _session_account_id: &AccountId, _new_expiration: &RefreshPairExpirationSeconds) -> Fallible<(), RefreshSessionSeriesError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn session_series_to_be_refreshed() {
        let result = MockRefreshSessionSeries.try_refresh_session_series(
            &*SESSION_SERIES_TO_BE_REFRESHED,
            &AccountId::new(Uuid7::now()),
            &RefreshPairExpirationSeconds::new(1),
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn session_series_to_not_be_refreshed() {
        let result = MockRefreshSessionSeries.try_refresh_session_series(
            &SessionSeries::gen(),
            &AccountId::new(Uuid7::now()),
            &RefreshPairExpirationSeconds::new(1),
        ).await;
        assert!(result.is_ok());
    }
}