use std::{str::FromStr, sync::{Arc, LazyLock}};

use bb8_redis::redis::cmd;
use scylla::{prepared_statement::PreparedStatement, Session};
use thiserror::Error;
use uuid::Uuid;

use crate::{common::{email::{address::Email, resend::ResendEmailSender, send::{Body, EmailSender, HtmlContent, NetmateEmail, PlainText, SenderName, Subject}}, fallible::Fallible, id::{uuid7::Uuid7, AccountId}, language::Language, session::value::{RefreshToken, SessionId, SessionSeries, REFRESH_PAIR_SEPARATOR}, unixtime::UnixtimeMillis}, helper::{error::InitError, scylla::prepare, valkey::{conn, Pool}}, translation::ja};

use super::dsl::{authenticate::{AuthenticateSession, AuthenticateSessionError}, extract_session_info::ExtractSessionInformation, manage_session::ManageSession, reauthenticate::{ReAuthenticateSession, ReAuthenticateSessionError}, update_refresh_token::RefreshTokenExpirationSeconds, update_session::SessionExpirationSeconds};


pub struct ManageSessionInterpreter {
    db: Arc<Session>,
    cache: Arc<Pool>,
    select_email_and_language: Arc<PreparedStatement>,
    select_last_series_id_extension_time: Arc<PreparedStatement>,
    update_series_id_expiration: Arc<PreparedStatement>,
    delete_all_sessions: Arc<PreparedStatement>
}

const SESSION_EXPIRATION: SessionExpirationSeconds = SessionExpirationSeconds::new(30 * 60);
const REFRESH_TOKEN_EXPIRATION: RefreshTokenExpirationSeconds = RefreshTokenExpirationSeconds::new(30 * 24 * 60 * 60);

impl ManageSession for ManageSessionInterpreter {
    fn session_expiration() -> &'static SessionExpirationSeconds {
        &SESSION_EXPIRATION
    }

    fn refresh_token_expiration() -> &'static RefreshTokenExpirationSeconds {
        &REFRESH_TOKEN_EXPIRATION
    }
}

impl ExtractSessionInformation for ManageSessionInterpreter {}

const SESSION_ID_NAMESPACE: &str = "sid";

impl AuthenticateSession for ManageSessionInterpreter {
    async fn resolve_session_id_to_account_id(&self, session_id: &SessionId) -> Fallible<Option<AccountId>, AuthenticateSessionError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> AuthenticateSessionError {
            AuthenticateSessionError::ResolveSessionIdFailed(e.into())
        }
        
        let mut conn = conn(&self.cache, handle_error)
            .await?;

        let key = format!("{}:{}", SESSION_ID_NAMESPACE, session_id.to_string());

        cmd("GET")
            .arg(key)
            .query_async::<Option<Uuid>>(&mut *conn)
            .await
            .map_err(handle_error)?
            .map(|uuid| Uuid7::try_from(uuid))
            .transpose()
            .map_or_else(|e| Err(handle_error(e)), |o| Ok(o.map(AccountId::new)))
    }
}

const REFRESH_PAIR_NAMESPACE: &str = "rfp";
const REFRESH_PAIR_VALUE_SEPARATOR: &str = "$";

impl ReAuthenticateSession for ManageSessionInterpreter {
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
                    .map(|u| AccountId::new(u))?;

                Ok((token, account_id))
            }))
            .transpose()
    }
}


/*#[derive(Debug, Clone)]
pub struct ManageSessionImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    select_email_and_language: Arc<PreparedStatement>,
    select_last_series_id_extension_time: Arc<PreparedStatement>,
    update_series_id_expiration: Arc<PreparedStatement>,
    delete_all_sessions: Arc<PreparedStatement>
}

impl ManageSessionImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<Self>> {
        let select_email_and_language = prepare::<InitError<Self>>(
            &db,
            "SELECT email, language FROM accounts WHERE id = ?"
        ).await?;

        let select_last_series_id_extension_time = prepare::<InitError<Self>>(
            &db,
            "SELECT updated_at FROM login_ids WHERE account_id = ? AND series_id = ?"
        ).await?;

        let update_series_id_expiration = prepare::<InitError<Self>>(
            &db,
            "UPDATE login_ids SET updated_at = ? WHERE account_id = ? AND series_id = ? TTL 34560000"
        ).await?;

        let delete_all_sessions = prepare::<InitError<Self>>(
            &db,
            "DELETE FROM login_ids WHERE account_id = ?"
        ).await?;

        Ok(Self { db, cache, select_email_and_language, select_last_series_id_extension_time, update_series_id_expiration, delete_all_sessions })
    }
}

const SESSION_MANAGEMENT_ID_CACHE_NAMESPACE: &str = "smid";
const LOGIN_SERIES_ID_CACHE_NAMESPACE: &str = "lsid";
const LOGIN_SERIES_ID_CACHE_SEPARATOR: &str = "$";

const SESSION_EXTENSION_THRESHOLD: u64 = 30 * 24 * 60 * 60 * 1000;

const SECURITY_EMAIL_ADDRESS: LazyLock<NetmateEmail> = LazyLock::new(|| NetmateEmail::try_from(Email::from_str("security@account.netmate.app").unwrap()).unwrap());
const SECURITY_NOTIFICATION_SUBJECT: LazyLock<Subject> = LazyLock::new(|| Subject::from_str(ja::session::SECURITY_NOTIFICATION_SUBJECT).unwrap());

impl ManageSession for ManageSessionImpl {
    async fn get_login_token_and_account_id(&self, series_id: &SessionSeries) -> Fallible<(Option<RefreshToken>, Option<AccountId>), ManageSessionError> {
        let mut conn = self.cache
            .get()
            .await
            .map_err(|e| ManageSessionError::GetLoginTokenAndAccountIdFailed(e.into()))?;

        let res = cmd("GET")
            .arg(format!("{}:{}", LOGIN_SERIES_ID_CACHE_NAMESPACE, series_id.value().value()))
            .query_async::<Option<String>>(&mut *conn)
            .await
            .map_err(|e| ManageSessionError::GetLoginTokenAndAccountIdFailed(e.into()))?;

        match res {
            Some(s) => {
                let mut parts = s.splitn(2, LOGIN_SERIES_ID_CACHE_SEPARATOR);
                let token = parts.next()
                    .and_then(|s| RefreshToken::from_str(s).ok());
                let account_id = parts.next()
                    .and_then(|s| Uuid::from_str(s).ok())
                    .and_then(|u| Uuid7::try_from(u).ok())
                    .map(AccountId::new);
                Ok((token, account_id))
            },
            None => Ok((None, None))
        }
    }

    async fn register_new_session_management_id_with_account_id(&self, new_session_management_id: &SessionId, account_id: &AccountId) -> Fallible<(), ManageSessionError> {
        let mut conn = self.cache
            .get()
            .await
            .map_err(|e| ManageSessionError::RegisteredNewSessionManagementIdFailed(e.into()))?;

        cmd("SET")
            .arg(format!("{}:{}", SESSION_MANAGEMENT_ID_CACHE_NAMESPACE, new_session_management_id.value().value()))
            .arg(account_id.value().value().to_string())
            .exec_async(&mut *conn)
            .await
            .map_err(|e| ManageSessionError::RegisteredNewSessionManagementIdFailed(e.into()))
    }

    async fn register_new_login_id_with_account_id(&self, login_series_id: &SessionSeries, new_login_token: &RefreshToken, account_id: &AccountId) -> Fallible<(), ManageSessionError> {
        let mut conn = self.cache
            .get()
            .await
            .map_err(|e| ManageSessionError::RegisteredNewLoginIdFailed(e.into()))?;

        cmd("SET")
            .arg(format!("{}:{}", LOGIN_SERIES_ID_CACHE_NAMESPACE, login_series_id.value().value()))
            .arg(format!("{}{}{}", new_login_token.value().value(), REFRESH_PAIR_SEPARATOR, account_id.value().value()))
            .exec_async(&mut *conn)
            .await
            .map_err(|e| ManageSessionError::RegisteredNewLoginIdFailed(e.into()))
    }

    async fn get_last_series_id_extension_time(&self, account_id: &AccountId, series_id: &SessionSeries) -> Fallible<SeriesIdRefreshTimestamp, ManageSessionError> {
        self.db
            .execute(&self.select_last_series_id_extension_time, (account_id.value().value(), series_id.value().value()))
            .await
            .map_err(|e| ManageSessionError::GetLastSeriesIdExtensionTimeFailed(e.into()))?
            .first_row_typed::<(i64, )>()
            .map_err(|e| ManageSessionError::GetLastSeriesIdExtensionTimeFailed(e.into()))
            .map(|(updated_at, )| SeriesIdRefreshTimestamp::new(UnixtimeMillis::new(updated_at as u64)))
    }

    fn should_extend_series_id_expiration(last_refreshed_at: &SeriesIdRefreshTimestamp) -> bool {
        UnixtimeMillis::now().value() - last_refreshed_at.value().value() > SESSION_EXTENSION_THRESHOLD
    }

    async fn extend_series_id_expiration(&self, account_id: &AccountId, series_id: &SessionSeries) -> Fallible<(), ManageSessionError> {
        let now = UnixtimeMillis::now()
            .value() as i64;

        self.db
            .execute(&self.update_series_id_expiration, (now, account_id.value().value(), series_id.value().value()))
            .await
            .map(|_| ())
            .map_err(|e| ManageSessionError::ExtendSeriesIdExpirationFailed(e.into()))
    }

    async fn delete_all_sessions(&self, account_id: &AccountId) -> Fallible<(), ManageSessionError> {
        self.db
            .execute(&self.delete_all_sessions, (account_id.value().value(),))
            .await
            .map(|_| ())
            .map_err(|e| ManageSessionError::DeleteAllSessionsFailed(e.into()))
    }

    async fn get_email_and_language(&self, account_id: &AccountId) -> Fallible<(Email, Language), ManageSessionError> {
        self.db
            .execute(&self.select_email_and_language, (account_id.value().value(),))
            .await
            .map_err(|e| ManageSessionError::GetEmailAndLanguageFailed(e.into()))?
            .first_row_typed::<(String, i8)>()
            .map_err(|e| ManageSessionError::GetEmailAndLanguageFailed(e.into()))
            .and_then(|(email, language)| {
                let email = Email::from_str(&email)
                    .map_err(|e| ManageSessionError::GetEmailAndLanguageFailed(e.into()))?;
                let language = Language::try_from(language)
                    .map_err(|e| ManageSessionError::GetEmailAndLanguageFailed(e.into()))?;
                Ok((email, language))
            })
    }

    async fn send_security_notification_email(&self, email: &Email, language: &Language) -> Fallible<(), ManageSessionError> {
        let (subject, html_content, plain_text) = match language {
            _ => (&*SECURITY_NOTIFICATION_SUBJECT, ja::session::SECURITY_NOTIFICATION_BODY_HTML, ja::session::SECURITY_NOTIFICATION_BODY_PLAIN)
        };

        let body = Body::new(HtmlContent::new(html_content), PlainText::new(plain_text));

        ResendEmailSender::send(&*SECURITY_EMAIL_ADDRESS, email, &SenderName::by(language), subject, &body)
            .await
            .map_err(|e| ManageSessionError::SendSecurityNotificationEmailFailed(e.into()))
    }
}*/