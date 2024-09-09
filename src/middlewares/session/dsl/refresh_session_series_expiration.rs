use thiserror::Error;

use crate::common::fallible::Fallible;

pub(crate) trait RefreshSessionSeriesExpiration {
    async fn refresh_session_series_expiration() -> Fallible<(), UpdateSessionSeriesError>;
}

#[derive(Debug, Error)]
pub enum UpdateSessionSeriesError {
    #[error("セッションシリーズの更新に失敗しました")]
    UpdateSessionSeriesFailed(#[source] anyhow::Error),
}