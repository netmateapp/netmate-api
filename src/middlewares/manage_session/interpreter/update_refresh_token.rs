use bb8_redis::redis::cmd;

use crate::{common::{fallible::Fallible, id::AccountId, session::value::{RefreshToken, SessionSeries}}, helper::valkey::conn, middlewares::manage_session::{dsl::{manage_session::RefreshPairExpirationSeconds, update_refresh_token::{UpdateRefreshToken, UpdateRefreshTokenError}}, interpreter::{REFRESH_PAIR_NAMESPACE, REFRESH_PAIR_VALUE_SEPARATOR}}};

use super::ManageSessionImpl;

impl UpdateRefreshToken for ManageSessionImpl {
    async fn assign_new_refresh_token_with_expiration(&self, new_refresh_token: &RefreshToken, session_series: &SessionSeries, session_account_id: &AccountId, expiration: &RefreshPairExpirationSeconds) -> Fallible<(), UpdateRefreshTokenError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> UpdateRefreshTokenError {
            UpdateRefreshTokenError::AssignNewRefreshTokenFailed(e.into())
        }

        let mut conn = conn(&self.cache, handle_error)
            .await?;

        let key = format!("{}:{}", REFRESH_PAIR_NAMESPACE, session_series.to_string());
        let value = format!("{}{}{}", new_refresh_token.to_string(), REFRESH_PAIR_VALUE_SEPARATOR, session_account_id.to_string());

        cmd("SET")
            .arg(key)
            .arg(value)
            .arg("EX")
            .arg(expiration.as_secs())
            .exec_async(&mut *conn)
            .await
            .map_err(handle_error)
    }
}