use std::sync::Arc;

use refresh_session_series::{SelectLastSessionSeriesRefreshedAt, UpdateSessionSeriesTtl, SELECT_LAST_API_KEY_REFRESHED_AT, UPDATE_SESSION_SERIES_TTL};
use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{cql, helper::{error::InitError, scylla::{prep, prepare}, valkey::Pool}};

use super::dsl::{extract_session_info::ExtractSessionInformation, manage_session::{ManageSession, RefreshPairExpirationSeconds, SessionExpirationSeconds}, set_cookie::SetSessionCookie};

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
    select_last_session_series_refreshed_at: SelectLastSessionSeriesRefreshedAt,
    update_session_series_ttl: UpdateSessionSeriesTtl,
    select_email_and_language: Arc<PreparedStatement>,
    select_all_session_series: Arc<PreparedStatement>,
    delete_all_session_series: Arc<PreparedStatement>
}

impl ManageSessionImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<Self>> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> InitError<ManageSessionImpl> {
            InitError::new(e.into())
        }

        let select_last_session_series_refreshed_at = prepare(&db, SelectLastSessionSeriesRefreshedAt, SELECT_LAST_API_KEY_REFRESHED_AT)
            .await
            .map_err(handle_error)?;

        let update_session_series_ttl = prepare(&db, UpdateSessionSeriesTtl, UPDATE_SESSION_SERIES_TTL)
            .await
            .map_err(handle_error)?;

        let select_email_and_language = prep::<InitError<Self>>(
            &db,
            cql!("SELECT email, language FROM accounts WHERE id = ? LIMIT 1")
        ).await?;

        let select_all_session_series = prep::<InitError<Self>>(
            &db,
            cql!("SELECT FROM session_series WHERE account_id = ?")
        ).await?;

        let delete_all_session_series = prep::<InitError<Self>>(
            &db,
            cql!("DELETE FROM login_ids WHERE account_id = ?")
        ).await?;

        Ok(Self { db, cache, select_last_session_series_refreshed_at, update_session_series_ttl, select_email_and_language, select_all_session_series, delete_all_session_series })
    }
}

const SESSION_ID_NAMESPACE: &str = "sid";
const SESSION_EXPIRATION: SessionExpirationSeconds = SessionExpirationSeconds::new(30 * 60);

const REFRESH_PAIR_NAMESPACE: &str = "rfp";
const REFRESH_PAIR_EXPIRATION: RefreshPairExpirationSeconds = RefreshPairExpirationSeconds::new(400 * 24 * 60 * 60);
const REFRESH_PAIR_VALUE_SEPARATOR: &str = "$";

impl ManageSession for ManageSessionImpl {
    fn session_expiration() -> &'static SessionExpirationSeconds {
        &SESSION_EXPIRATION
    }

    fn refresh_pair_expiration() -> &'static RefreshPairExpirationSeconds {
        &REFRESH_PAIR_EXPIRATION
    }
}

impl ExtractSessionInformation for ManageSessionImpl {}

impl SetSessionCookie for ManageSessionImpl {}
