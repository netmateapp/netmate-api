use redis::cmd;

use crate::{common::{fallible::Fallible, id::account_id::AccountId, session::{refresh_pair_expiration::RefreshPairExpirationSeconds, refresh_token::RefreshToken, session_series::SessionSeries}, unixtime::UnixtimeMillis}, helper::redis::conn, middlewares::{session::{RefreshPairKey, RefreshPairValue}, start_session::dsl::assign_refresh_pair::{AssignRefreshPair, AssignRefreshPairError}}};

use super::StartSessionImpl;

impl AssignRefreshPair for StartSessionImpl {
    async fn try_assign_refresh_pair_with_expiration_if_unused(&self, session_series: &SessionSeries, refresh_token: &RefreshToken, session_account_id: AccountId, expiration: RefreshPairExpirationSeconds) -> Fallible<(), AssignRefreshPairError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> AssignRefreshPairError {
            AssignRefreshPairError::AssignRefreshPairFailed(e.into())
        }
        
        let mut conn = conn(&self.cache, handle_error).await?;

        cmd("SET")
            .arg(RefreshPairKey::new(session_series))
            .arg(RefreshPairValue::new(refresh_token, session_account_id))
            .arg("EX")
            .arg(expiration)
            .arg("NX")
            .query_async::<Option<()>>(&mut *conn)
            .await
            .map_err(handle_error)?
            .map_or_else(|| Err(AssignRefreshPairError::SessionSeriesAlreadyUsed), |_| Ok(()))?;

        self.db
            .execute_unpaged(&self.insert_session_series, (session_account_id, session_series, UnixtimeMillis::now(), expiration))
            .await
            .map(|_| ())
            .map_err(handle_error)
    }
}