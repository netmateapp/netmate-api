use std::{str::FromStr, sync::LazyLock};

use crate::{common::{email::{address::Email, resend::ResendEmailSender, send::{Body, EmailSender, HtmlContent, NetmateEmail, PlainText, SenderName, Subject}}, fallible::Fallible, id::AccountId, language::Language}, middlewares::manage_session::dsl::mitigate_session_theft::{MitigateSessionTheft, MitigateSessionTheftError}, translation::ja};

use super::ManageSessionImpl;

const SECURITY_EMAIL_ADDRESS: LazyLock<NetmateEmail> = LazyLock::new(|| NetmateEmail::try_from(Email::from_str("security@account.netmate.app").unwrap()).unwrap());
const SECURITY_NOTIFICATION_SUBJECT: LazyLock<Subject> = LazyLock::new(|| Subject::from_str(ja::session::SECURITY_NOTIFICATION_SUBJECT).unwrap());

impl MitigateSessionTheft for ManageSessionImpl {
    async fn fetch_email_and_language(&self, account_id: &AccountId) -> Fallible<(Email, Language), MitigateSessionTheftError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> MitigateSessionTheftError {
            MitigateSessionTheftError::FetchEmailAndLanguageFailed(e.into())
        }

        self.db
            .execute_unpaged(&self.select_email_and_language, (account_id.to_string(),))
            .await
            .map_err(handle_error)?
            .first_row_typed::<(String, i8)>()
            .map_err(handle_error)
            .and_then(|(email, language)| {
                let email = Email::from_str(email.as_str())
                    .map_err(handle_error)?;
                let language = Language::try_from(language)
                    .map_err(handle_error)?;
                Ok((email, language))
            })
    }

    async fn send_security_notification(&self, email: &Email, language: &Language) -> Fallible<(), MitigateSessionTheftError> {
        let (subject, html_content, plain_text) = match language {
            _ => (&*SECURITY_NOTIFICATION_SUBJECT, ja::session::SECURITY_NOTIFICATION_BODY_HTML, ja::session::SECURITY_NOTIFICATION_BODY_PLAIN)
        };

        let body = Body::new(HtmlContent::new(html_content), PlainText::new(plain_text));

        ResendEmailSender::send(&*SECURITY_EMAIL_ADDRESS, email, &SenderName::by(language), subject, &body)
            .await
            .map_err(|e| MitigateSessionTheftError::SendSecurityNotificationFailed(e.into()))
    }

    async fn purge_all_session_series(&self, account_id: &AccountId) -> Fallible<(), MitigateSessionTheftError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> MitigateSessionTheftError {
            MitigateSessionTheftError::DeleteAllSessionSeriesFailed(e.into())
        }

        /*let all_session_series = self.db
            .execute_iter(&self.select_all_session_series, (account_id.to_string(), ))
            .await
            .map_err(handle_error)?
            .into_typed::<(String, )>()*/
            
            

        self.db
            .execute_unpaged(&self.delete_all_session_series, (account_id.to_string(), ))
            .await
            .map(|_| ())
            .map_err(handle_error)
    }
}
