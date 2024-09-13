use std::{str::FromStr, sync::{Arc, LazyLock}};

use redis::cmd;
use scylla::{prepared_statement::PreparedStatement, transport::session::TypedRowIter, FromRow, Session};

use crate::{common::{email::{address::Email, resend::ResendEmailSender, send::{Body, EmailSender, HtmlContent, NetmateEmail, PlainText, SenderName, Subject}}, fallible::Fallible, id::AccountId, language::Language, session::value::SessionSeries}, helper::{scylla::{Statement, TypedStatement, Unit}, valkey::conn}, middlewares::manage_session::{dsl::mitigate_session_theft::{MitigateSessionTheft, MitigateSessionTheftError}, interpreter::SESSION_ID_NAMESPACE}, translation::ja};

use super::ManageSessionImpl;

const SECURITY_EMAIL_ADDRESS: LazyLock<NetmateEmail> = LazyLock::new(|| NetmateEmail::try_from(Email::from_str("security@account.netmate.app").unwrap()).unwrap());
const SECURITY_NOTIFICATION_SUBJECT: LazyLock<Subject> = LazyLock::new(|| Subject::from_str(ja::session::SECURITY_NOTIFICATION_SUBJECT).unwrap());

impl MitigateSessionTheft for ManageSessionImpl {
    async fn fetch_email_and_language(&self, account_id: &AccountId) -> Fallible<(Email, Language), MitigateSessionTheftError> {
        self.select_email_and_language
            .query(&self.db, (account_id, ))
            .await
            .map_err(|e| MitigateSessionTheftError::FetchEmailAndLanguageFailed(e.into()))
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

        let all_session_series_key: &[String] = &self.select_all_session_series
            .query(&self.db, (account_id, ))
            .await
            .map_err(handle_error)?
            .filter(Result::is_ok)
            .map(Result::unwrap)
            .map(|(session_series, )| session_series)
            .map(|session_series| format!("{}:{}", SESSION_ID_NAMESPACE, session_series.to_string()))
            .collect::<Vec<String>>();

        self.delete_all_session_series
            .execute(&self.db, (account_id, ))
            .await
            .map(|_| ())
            .map_err(handle_error)?;

        // データベースでの削除後にキャッシュを削除するのは、エラー委譲でデータベースの削除をキャンセルしないため
        let mut conn = conn(&self.cache, handle_error).await?;

        cmd("DEL")
            .arg(all_session_series_key)
            .exec_async(&mut *conn)
            .await
            .map_err(handle_error)
    }
}


// 以下、型付きCQL文の定義
pub const SELECT_EMAIL_AND_LANGUAGE: Statement<SelectEmailAndLanguage>
    = Statement::of("SELECT email, language FROM accounts WHERE id = ? LIMIT 1");

#[derive(Debug)]
pub struct SelectEmailAndLanguage(pub Arc<PreparedStatement>);

impl<'a> TypedStatement<(&'a AccountId, ), (Email, Language)> for SelectEmailAndLanguage {
    type Result<U> = U where U: FromRow;

    async fn query(&self, db: &Arc<Session>, values: (&'a AccountId, )) -> anyhow::Result<(Email, Language)> {
        db.execute_unpaged(&self.0, values)
            .await
            .map_err(anyhow::Error::from)?
            .first_row_typed()
            .map_err(anyhow::Error::from)
    }
}

pub const SELECT_ALL_SESSION_SERIES: Statement<SelectAllSessionSeries>
    = Statement::of("SELECT FROM session_series WHERE account_id = ?");

#[derive(Debug)]
pub struct SelectAllSessionSeries(pub Arc<PreparedStatement>);

impl<'a> TypedStatement<(&'a AccountId, ), (SessionSeries, )> for SelectAllSessionSeries {
    type Result<U> = TypedRowIter<U> where U: FromRow;

    async fn query(&self, db: &Arc<Session>, values: (&'a AccountId, )) -> anyhow::Result<Self::Result<(SessionSeries, )>> {
        db.execute_unpaged(&self.0, values)
            .await
            .map_err(anyhow::Error::from)?
            .rows_typed()
            .map_err(anyhow::Error::from)
    }
}

pub const DELETE_ALL_SESSION_SERIES: Statement<DeleteAllSessionSeries>
    = Statement::of("DELETE FROM session_series WHERE account_id = ?");

#[derive(Debug)]
pub struct DeleteAllSessionSeries(pub Arc<PreparedStatement>);

impl<'a> TypedStatement<(&'a AccountId, ), Unit> for DeleteAllSessionSeries {
    type Result<U> = U where U: FromRow;

    async fn query(&self, db: &Arc<Session>, values: (&'a AccountId, )) -> anyhow::Result<Unit> {
        db.execute_unpaged(&self.0, values)
            .await
            .map(|_| Unit)
            .map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::{helper::scylla::{check_cql_query_type, check_cql_statement_type}, middlewares::manage_session::interpreter::SELECT_EMAIL_AND_LANGUAGE};

    use super::{DELETE_ALL_SESSION_SERIES, SELECT_ALL_SESSION_SERIES};

    #[test]
    fn check_select_email_and_language_type() {
        check_cql_query_type(SELECT_EMAIL_AND_LANGUAGE);
    }

    #[test]
    fn check_select_all_session_series_type() {
        check_cql_query_type(SELECT_ALL_SESSION_SERIES);
    }

    #[test]
    fn check_delete_all_session_series_type() {
        check_cql_statement_type(DELETE_ALL_SESSION_SERIES);
    }
}