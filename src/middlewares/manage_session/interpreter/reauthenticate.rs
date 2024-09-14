use std::str::FromStr;

use bb8_redis::redis::cmd;
use thiserror::Error;
use uuid::Uuid;

use crate::{common::{fallible::Fallible, id::{uuid7::Uuid7, AccountId}, session::value::{RefreshToken, SessionSeries}}, helper::redis::conn, middlewares::manage_session::{dsl::reauthenticate::{ReAuthenticateSession, ReAuthenticateSessionError}, interpreter::{REFRESH_PAIR_NAMESPACE, REFRESH_PAIR_VALUE_SEPARATOR}}};

use super::ManageSessionImpl;


impl ReAuthenticateSession for ManageSessionImpl {
    async fn fetch_refresh_token_and_account_id(&self, session_series: &SessionSeries) -> Fallible<Option<(RefreshToken, AccountId)>, ReAuthenticateSessionError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> ReAuthenticateSessionError {
            ReAuthenticateSessionError::FetchRefreshTokenAndAccountIdFailed(e.into())
        }

        #[derive(Debug, Error)]
        #[error("リフレッシュペア値の解析に失敗しました")]
        struct ParseRefreshPairValueError;

        let mut conn = conn(&self.cache, handle_error)
            .await?;

        let key = format!("{}:{}", REFRESH_PAIR_NAMESPACE, session_series.to_string());

        cmd("GET")
            .arg(key)
            .query_async::<Option<String>>(&mut *conn)
            .await
            .map_err(handle_error)
            .transpose()
            .map(|o| o.and_then(|s| {
                let mut parts = s.splitn(2, REFRESH_PAIR_VALUE_SEPARATOR);

                let token = parts.next()
                    .ok_or_else(|| handle_error(ParseRefreshPairValueError))
                    .map(|s| RefreshToken::from_str(s))?
                    .map_err(handle_error)?;

                let account_id = parts.next()
                    .ok_or_else(|| handle_error(ParseRefreshPairValueError))
                    .map(|s| Uuid::from_str(s))?
                    .map_err(handle_error)
                    .map(|u| Uuid7::try_from(u))?
                    .map_err(handle_error)
                    .map(|u| AccountId::of(u))?;

                Ok((token, account_id))
            }))
            .transpose()
    }
}