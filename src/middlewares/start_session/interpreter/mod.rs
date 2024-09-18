use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::helper::{error::InitError, redis::Pool, scylla::prepare};

use super::dsl::start_session::StartSession;

mod assign_refresh_pair;
mod assign_session_id;

#[derive(Debug)]
pub struct StartSessionImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    insert_session_series: Arc<PreparedStatement>,
}

impl StartSessionImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<Self>> {
        let insert_session_series = prepare(&db, "INSERT INTO session_series (account_id, series, refreshed_at) VALUES (?, ?, ?) USING TTL ?").await?;

        Ok(Self { db, cache, insert_session_series })
    }
}

impl StartSession for StartSessionImpl {}