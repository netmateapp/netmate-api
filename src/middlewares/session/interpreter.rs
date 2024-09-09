use std::{str::FromStr, sync::{Arc, LazyLock}};

use bb8_redis::redis::cmd;
use scylla::{frame::value::CqlTimestamp, prepared_statement::PreparedStatement, Session};
use thiserror::Error;
use uuid::Uuid;

use crate::{common::{email::{address::Email, resend::ResendEmailSender, send::{Body, EmailSender, HtmlContent, NetmateEmail, PlainText, SenderName, Subject}}, fallible::Fallible, id::{uuid7::Uuid7, AccountId}, language::Language, session::value::{RefreshToken, SessionId, SessionSeries, REFRESH_PAIR_SEPARATOR}, unixtime::UnixtimeMillis}, helper::{error::InitError, scylla::prepare, valkey::{conn, Pool}}, translation::ja};

use super::dsl::{authenticate::{AuthenticateSession, AuthenticateSessionError}, extract_session_info::ExtractSessionInformation, manage_session::ManageSession, reauthenticate::{ReAuthenticateSession, ReAuthenticateSessionError}, refresh_session_series::{LastSessionSeriesRefreshedTime, RefreshSessionSeries, RefreshSessionSeriesError, RefreshSessionSeriesThereshold}, set_cookie::SetSessionCookie, update_refresh_token::{RefreshPairExpirationSeconds, UpdateRefreshToken, UpdateRefreshTokenError}, update_session::{SessionExpirationSeconds, UpdateSession, UpdateSessionError}};


pub struct ManageSessionInterpreter {
    db: Arc<Session>,
    cache: Arc<Pool>,
    select_email_and_language: Arc<PreparedStatement>,
    select_last_session_series_refreshed_at: Arc<PreparedStatement>,
    update_session_series_ttl: Arc<PreparedStatement>,
    delete_all_session_series: Arc<PreparedStatement>
}

impl ManageSessionInterpreter {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<Self>> {
        let select_email_and_language = prepare::<InitError<Self>>(
            &db,
            "SELECT email, language FROM accounts WHERE id = ? LIMIT 1"
        ).await?;

        let select_last_session_series_refreshed_at = prepare::<InitError<Self>>(
            &db,
            "SELECT refreshed_at FROM session_series WHERE account_id = ? AND series = ? LIMIT 1"
        ).await?;

        let update_session_series_ttl = prepare::<InitError<Self>>(
            &db,
            "UPDATE session_series SET refreshed_at = ? WHERE account_id = ? AND series = ? USING TTL ?"
        ).await?;

        let delete_all_session_series = prepare::<InitError<Self>>(
            &db,
            "DELETE FROM login_ids WHERE account_id = ?"
        ).await?;

        Ok(Self { db, cache, select_email_and_language, select_last_session_series_refreshed_at, update_session_series_ttl, delete_all_session_series })
    }
}

const SESSION_ID_NAMESPACE: &str = "sid";
const SESSION_EXPIRATION: SessionExpirationSeconds = SessionExpirationSeconds::new(30 * 60);

const REFRESH_PAIR_NAMESPACE: &str = "rfp";
const REFRESH_TOKEN_EXPIRATION: RefreshPairExpirationSeconds = RefreshPairExpirationSeconds::new(400 * 24 * 60 * 60);
const REFRESH_PAIR_VALUE_SEPARATOR: &str = "$";

impl ManageSession for ManageSessionInterpreter {
    fn session_expiration() -> &'static SessionExpirationSeconds {
        &SESSION_EXPIRATION
    }

    fn refresh_pair_expiration() -> &'static RefreshPairExpirationSeconds {
        &REFRESH_TOKEN_EXPIRATION
    }
}

impl ExtractSessionInformation for ManageSessionInterpreter {}

impl SetSessionCookie for ManageSessionInterpreter {}

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

impl UpdateSession for ManageSessionInterpreter {
    async fn try_assign_new_session_id_with_expiration_if_unused(&self, new_session_id: &SessionId, session_account_id: &AccountId, new_expiration: &SessionExpirationSeconds) -> Fallible<(), UpdateSessionError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> UpdateSessionError {
            UpdateSessionError::AssignNewSessionIdFailed(e.into())
        }

        let mut conn = conn(&self.cache, handle_error)
            .await?;

        let key = format!("{}:{}", SESSION_ID_NAMESPACE, new_session_id.to_string());

        cmd("SET")
            .arg(key)
            .arg(session_account_id.to_string())
            .arg("EX")
            .arg(new_expiration.as_secs())
            .arg("NX")
            .query_async::<Option<()>>(&mut *conn)
            .await
            .map_err(handle_error)?
            .map_or_else(|| Err(UpdateSessionError::SessionIdAlreadyUsed), |_| Ok(()))
    }
}

impl UpdateRefreshToken for ManageSessionInterpreter {
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

const REFRESH_SESSION_SERIES_THERESHOLD: RefreshSessionSeriesThereshold = RefreshSessionSeriesThereshold::days(30);

impl RefreshSessionSeries for ManageSessionInterpreter {
    async fn fetch_last_session_series_refreshed_at(&self, session_series: &SessionSeries, session_account_id: &AccountId) -> Fallible<LastSessionSeriesRefreshedTime, RefreshSessionSeriesError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> RefreshSessionSeriesError {
            RefreshSessionSeriesError::FetchLastSessionSeriesRefreshedAtFailed(e.into())
        }

        self.db
            .execute_unpaged(&self.select_last_session_series_refreshed_at, (session_account_id.value().value(), session_series.value().value()))
            .await
            .map_err(handle_error)?
            .first_row_typed::<(CqlTimestamp, )>()
            .map_err(handle_error)
            .map(|(refreshed_at, )| LastSessionSeriesRefreshedTime::new(UnixtimeMillis::from(refreshed_at.0)))
    }

    fn refresh_thereshold() -> &'static RefreshSessionSeriesThereshold {
        &REFRESH_SESSION_SERIES_THERESHOLD
    }

    async fn refresh_session_series(&self, session_series: &SessionSeries, session_account_id: &AccountId, new_expiration: &RefreshPairExpirationSeconds) -> Fallible<(), RefreshSessionSeriesError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> RefreshSessionSeriesError {
            RefreshSessionSeriesError::RefreshSessionSeriesFailed(e.into())
        }

        let values = (
            session_account_id.to_string(),
            session_series.to_string(),
            i64::from(UnixtimeMillis::now()),
            i32::from(new_expiration.clone())
        );

        self.db
            .execute_unpaged(&self.update_session_series_ttl, values)
            .await
            .map(|_| ())
            .map_err(handle_error)
    }
}



/*
const SECURITY_EMAIL_ADDRESS: LazyLock<NetmateEmail> = LazyLock::new(|| NetmateEmail::try_from(Email::from_str("security@account.netmate.app").unwrap()).unwrap());
const SECURITY_NOTIFICATION_SUBJECT: LazyLock<Subject> = LazyLock::new(|| Subject::from_str(ja::session::SECURITY_NOTIFICATION_SUBJECT).unwrap());

impl ManageSession for ManageSessionImpl {
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