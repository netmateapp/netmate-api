use std::sync::Arc;

use redis::cmd;
use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, id::account_id::AccountId, session::session_series::SessionSeries}, helper::{error::InitError, redis::{conn, Pool}, scylla::prepare}, middlewares::session::RefreshPairKey};

use super::dsl::{SignOut, SignOutError};

pub struct SignOutImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    delete_session_series: Arc<PreparedStatement>,
}

impl SignOutImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<Self>> {
        let delete_session_series = prepare(&db, "DELETE FROM session_series WHERE account_id = ? AND series = ?").await?;

        Ok(Self { db, cache, delete_session_series })
    }
}

impl SignOut for SignOutImpl {
    async fn sign_out(&self, account_id: AccountId, session_series: &SessionSeries) -> Fallible<(), SignOutError> {
        let mut conn = conn(&self.cache, |e| SignOutError::SignOutFailed(e.into())).await?;

        cmd("DEL")
            .arg(RefreshPairKey::new(session_series))
            .exec_async(&mut *conn)
            .await
            .map_err(|e| SignOutError::SignOutFailed(e.into()))?;

        self.db
            .execute_unpaged(&self.delete_session_series, (account_id, session_series))
            .await
            .map(|_| ())
            .map_err(|e| SignOutError::SignOutFailed(e.into()))
    }
}