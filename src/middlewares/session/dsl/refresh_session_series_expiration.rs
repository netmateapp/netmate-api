use thiserror::Error;

use crate::common::{fallible::Fallible, id::AccountId, session::value::SessionSeries};

use super::update_refresh_token::RefreshPairExpirationSeconds;

pub(crate) trait RefreshSessionSeriesExpiration {
    async fn refresh_session_series_expiration(session_series: &SessionSeries, session_account_id: &AccountId, new_expiration: &RefreshPairExpirationSeconds) -> Fallible<(), UpdateSessionSeriesError>;
}

#[derive(Debug, Error)]
pub enum UpdateSessionSeriesError {
    #[error("セッションシリーズの更新に失敗しました")]
    UpdateSessionSeriesFailed(#[source] anyhow::Error),
}