use std::sync::Arc;

use assign_refresh_pair::{InsertSessionSeries, INSERT_SESSION_SERIES};
use scylla::Session;

use crate::helper::{error::InitError, redis::Pool};

use super::dsl::start_session::StartSession;

mod assign_refresh_pair;
mod assign_session_id;

pub struct StartSessionImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    insert_session_series: Arc<InsertSessionSeries>,
}

impl StartSessionImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<Self>> {
        let insert_session_series = INSERT_SESSION_SERIES.prepared(&db, InsertSessionSeries)
            .await
            .map_err(|e| InitError::new(e.into()))?;

        Ok(Self { db, cache, insert_session_series })
    }
}

impl StartSession for StartSessionImpl {}