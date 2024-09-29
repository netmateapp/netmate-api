use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::helper::{error::InitError, redis::connection::Pool, scylla::prepare};

use super::dsl::{extract_session_info::ExtractSessionInformation, manage_session::ManageSession};

mod authenticate;
mod mitigate_session_theft;
mod reauthenticate;
mod refresh_session_series;
mod update_refresh_token;
mod update_session;

#[derive(Debug)]
pub struct ManageSessionImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    select_last_session_series_refreshed_at: Arc<PreparedStatement>,
    update_session_series_ttl: Arc<PreparedStatement>,
    select_email_and_language: Arc<PreparedStatement>,
    select_all_session_series: Arc<PreparedStatement>,
    delete_all_session_series: Arc<PreparedStatement>,
}

impl ManageSessionImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<Self>> {
        let select_last_session_series_refreshed_at = prepare(&db, "SELECT refreshed_at FROM session_series WHERE account_id = ? AND series = ? LIMIT 1").await?;

        let update_session_series_ttl = prepare(&db, "UPDATE session_series SET refreshed_at = ? WHERE account_id = ? AND series = ? USING TTL ?").await?;

        let select_email_and_language = prepare(&db, "SELECT email, language FROM accounts WHERE id = ? LIMIT 1").await?;

        let select_all_session_series = prepare(&db, "SELECT FROM session_series WHERE account_id = ?").await?;

        let delete_all_session_series = prepare(&db, "DELETE FROM session_series WHERE account_id = ?").await?;

        Ok(Self { db, cache, select_last_session_series_refreshed_at, update_session_series_ttl, select_email_and_language, select_all_session_series, delete_all_session_series })
    }
}

impl ManageSession for ManageSessionImpl {}

impl ExtractSessionInformation for ManageSessionImpl {}