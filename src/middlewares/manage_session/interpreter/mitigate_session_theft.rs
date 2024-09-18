use std::{str::FromStr, sync::LazyLock};

use redis::cmd;

use crate::{common::{email::{address::Email, resend::ResendEmailSender, send::{Body, EmailSender, HtmlContent, NetmateEmail, PlainText, SenderName, Subject}}, fallible::Fallible, id::account_id::AccountId, language::Language, session::session_series::SessionSeries}, helper::redis::conn, middlewares::{manage_session::dsl::mitigate_session_theft::{MitigateSessionTheft, MitigateSessionTheftError}, session::RefreshPairKey}, translation::ja};

use super::ManageSessionImpl;

static SECURITY_EMAIL_ADDRESS: LazyLock<NetmateEmail> = LazyLock::new(|| NetmateEmail::try_from(Email::from_str("security@account.netmate.app").unwrap()).unwrap());
static SECURITY_NOTIFICATION_SUBJECT: LazyLock<Subject> = LazyLock::new(|| Subject::from_str(ja::session::SECURITY_NOTIFICATION_SUBJECT).unwrap());

impl MitigateSessionTheft for ManageSessionImpl {
    async fn fetch_email_and_language(&self, account_id: AccountId) -> Fallible<(Email, Language), MitigateSessionTheftError> {
        self.db
            .execute_unpaged(&self.select_email_and_language, (account_id, ))
            .await
            .map_err(|e| MitigateSessionTheftError::FetchEmailAndLanguageFailed(e.into()))?
            .first_row_typed::<(Email, Language)>()
            .map_err(|e| MitigateSessionTheftError::FetchEmailAndLanguageFailed(e.into()))
    }

    async fn send_security_notification(&self, email: &Email, language: Language) -> Fallible<(), MitigateSessionTheftError> {
        let (subject, html_content, plain_text) = match language {
            _ => (&*SECURITY_NOTIFICATION_SUBJECT, ja::session::SECURITY_NOTIFICATION_BODY_HTML, ja::session::SECURITY_NOTIFICATION_BODY_PLAIN)
        };

        let body = Body::new(HtmlContent::new(html_content), PlainText::new(plain_text));

        ResendEmailSender::send(&SECURITY_EMAIL_ADDRESS, email, &SenderName::by(language), subject, &body)
            .await
            .map_err(|e| MitigateSessionTheftError::SendSecurityNotificationFailed(e.into()))
    }

    async fn purge_all_session_series(&self, account_id: AccountId) -> Fallible<(), MitigateSessionTheftError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> MitigateSessionTheftError {
            MitigateSessionTheftError::DeleteAllSessionSeriesFailed(e.into())
        }

        let all_session_series = self.db
            .execute_unpaged(&self.select_all_session_series, (account_id, ))
            .await
            .map_err(handle_error)?
            .rows_typed::<(SessionSeries, )>()
            .map(|rows| {
                rows.flatten()
                    .map(|(session_series, )| session_series)
                    .map(|session_series| RefreshPairKey::new(&session_series))
                    .collect::<Vec<RefreshPairKey>>()
            })
            .map_err(handle_error)?;
        
        let mut conn = conn(&self.cache, handle_error).await?;

        cmd("DEL")
            .arg(all_session_series.as_slice())
            .query_async::<()>(&mut *conn)
            .await
            .map_err(handle_error)?;

        self.db
            .execute_unpaged(&self.delete_all_session_series, (account_id, ))
            .await
            .map(|_| ())
            .map_err(handle_error)
    }
}
