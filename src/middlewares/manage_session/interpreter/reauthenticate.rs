use std::str::FromStr;

use redis::cmd;
use thiserror::Error;
use uuid::Uuid;

use crate::{common::{fallible::Fallible, profile::account_id::AccountId, session::{refresh_token::RefreshToken, session_series::SessionSeries}, uuid::uuid7::Uuid7}, helper::redis::conn, middlewares::{manage_session::dsl::reauthenticate::{ReAuthenticateSession, ReAuthenticateSessionError}, session::{RefreshPairKey, REFRESH_PAIR_VALUE_SEPARATOR}}};

use super::ManageSessionImpl;

impl ReAuthenticateSession for ManageSessionImpl {
    async fn fetch_refresh_token_and_account_id(&self, session_series: &SessionSeries) -> Fallible<Option<(RefreshToken, AccountId)>, ReAuthenticateSessionError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> ReAuthenticateSessionError {
            ReAuthenticateSessionError::FetchRefreshTokenAndAccountIdFailed(e.into())
        }

        #[derive(Debug, Error)]
        #[error("リフレッシュペア値の解析に失敗しました")]
        struct ParseRefreshPairValueError;

        let mut conn = conn(&self.cache, handle_error).await?;
        
        cmd("GET")
            .arg(RefreshPairKey::new(session_series))
            .query_async::<Option<String>>(&mut *conn)
            .await
            .map_err(handle_error)
            .transpose()
            .map(|o| o.and_then(|s| {
                let mut parts = s.splitn(2, REFRESH_PAIR_VALUE_SEPARATOR);

                let token = parts.next()
                    .ok_or_else(|| handle_error(ParseRefreshPairValueError))
                    .map(RefreshToken::from_str)?
                    .map_err(handle_error)?;

                let account_id = parts.next()
                    .ok_or_else(|| handle_error(ParseRefreshPairValueError))
                    .map(Uuid::from_str)?
                    .map_err(handle_error)
                    .map(Uuid7::try_from)?
                    .map_err(handle_error)
                    .map(AccountId::of)?;

                Ok((token, account_id))
            }))
            .transpose()
    }
}
