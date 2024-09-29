use bb8_redis::redis::cmd;

use crate::{common::{fallible::Fallible, profile::account_id::AccountId, session::{refresh_pair_expiration::RefreshPairExpirationSeconds, refresh_token::RefreshToken, session_series::SessionSeries}}, helper::redis::conn, middlewares::{manage_session::dsl::update_refresh_token::{UpdateRefreshToken, UpdateRefreshTokenError}, session::{RefreshPairKey, RefreshPairValue}}};

use super::ManageSessionImpl;

impl UpdateRefreshToken for ManageSessionImpl {
    async fn assign_new_refresh_token_with_expiration(&self, new_refresh_token: &RefreshToken, session_series: &SessionSeries, session_account_id: AccountId, expiration: RefreshPairExpirationSeconds) -> Fallible<(), UpdateRefreshTokenError> {
        let mut conn = conn(&self.cache, |e| UpdateRefreshTokenError::AssignNewRefreshTokenFailed(e.into())).await?;

        cmd("SET")
            .arg(RefreshPairKey::new(session_series))
            .arg(RefreshPairValue::new(new_refresh_token, session_account_id))
            .arg("EX")
            .arg(expiration)
            .exec_async(&mut *conn)
            .await
            .map_err(|e| UpdateRefreshTokenError::AssignNewRefreshTokenFailed(e.into()))
    }
}