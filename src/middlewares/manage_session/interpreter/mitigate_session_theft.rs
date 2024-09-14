use std::{str::FromStr, sync::{Arc, LazyLock}};

use redis::{cmd, ToRedisArgs};
use scylla::{prepared_statement::PreparedStatement, FromRow, Session};

use crate::{common::{email::{address::Email, resend::ResendEmailSender, send::{Body, EmailSender, HtmlContent, NetmateEmail, PlainText, SenderName, Subject}}, fallible::Fallible, id::AccountId, language::Language, session::value::SessionSeries}, helper::{redis::{Connection, TypedCommand, DEL_COMMAND, NAMESPACE_SEPARATOR}, scylla::{Statement, TypedStatement, Unit}}, middlewares::manage_session::{dsl::mitigate_session_theft::{MitigateSessionTheft, MitigateSessionTheftError}, interpreter::SESSION_ID_NAMESPACE}, translation::ja};

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

        let all_session_series = self.select_all_session_series
            .query(&self.db, (account_id, ))
            .await
            .map_err(handle_error)?;

        let keys: Vec<Key<'_>> = all_session_series
            .iter()
            .map(|(session_series, )| Key(&session_series))
            .collect();

        DeleteAllSessionSeriesCommand.run(&self.cache, keys)
            .await
            .map_err(handle_error)?;

        self.delete_all_session_series
            .execute(&self.db, (account_id, ))
            .await
            .map(|_| ())
            .map_err(handle_error)
    }
}


// 以下、型付きCQL文の定義
pub const SELECT_EMAIL_AND_LANGUAGE: Statement<SelectEmailAndLanguage>
    = Statement::of("SELECT email, language FROM accounts WHERE id = ? LIMIT 1");

#[derive(Debug)]
pub struct SelectEmailAndLanguage(pub PreparedStatement);

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
pub struct SelectAllSessionSeries(pub PreparedStatement);

impl<'a> TypedStatement<(&'a AccountId, ), (SessionSeries, )> for SelectAllSessionSeries {
    type Result<U> = Vec<U> where U: FromRow;

    async fn query(&self, db: &Arc<Session>, values: (&'a AccountId, )) -> anyhow::Result<Self::Result<(SessionSeries, )>> {
        db.execute_unpaged(&self.0, values)
            .await
            .map_err(anyhow::Error::from)?
            .rows_typed()
            .map(|rows| {
                rows.filter(Result::is_ok)
                    .map(Result::unwrap)
                    .collect::<Vec<(SessionSeries, )>>()
        })
            .map_err(anyhow::Error::from)
    }
}

pub const DELETE_ALL_SESSION_SERIES: Statement<DeleteAllSessionSeries>
    = Statement::of("DELETE FROM session_series WHERE account_id = ?");

#[derive(Debug)]
pub struct DeleteAllSessionSeries(pub PreparedStatement);

impl<'a> TypedStatement<(&'a AccountId, ), Unit> for DeleteAllSessionSeries {
    type Result<U> = U where U: FromRow;

    async fn query(&self, db: &Arc<Session>, values: (&'a AccountId, )) -> anyhow::Result<Unit> {
        db.execute_unpaged(&self.0, values)
            .await
            .map(|_| Unit)
            .map_err(anyhow::Error::from)
    }
}

struct DeleteAllSessionSeriesCommand;

struct Key<'a>(&'a SessionSeries);

fn format_key(session_series: &SessionSeries) -> String {
    format!("{}{}{}", SESSION_ID_NAMESPACE, NAMESPACE_SEPARATOR, session_series)
}

impl<'a> ToRedisArgs for Key<'a> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite
    {
        format_key(self.0).write_redis_args(out);
    }
}

impl<'a> TypedCommand<Vec<Key<'a>>, ()> for DeleteAllSessionSeriesCommand {
    async fn execute(&self, mut conn: Connection<'_>, keys: Vec<Key<'a>>) -> anyhow::Result<()> {
        cmd(DEL_COMMAND).arg(keys)
            .query_async::<()>(&mut *conn)
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use crate::{common::session::value::SessionSeries, helper::{redis::NAMESPACE_SEPARATOR, scylla::{check_cql_query_type, check_cql_statement_type}}, middlewares::manage_session::interpreter::SELECT_EMAIL_AND_LANGUAGE};

    use super::{format_key, DELETE_ALL_SESSION_SERIES, SELECT_ALL_SESSION_SERIES};

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

    #[test]
    fn test_format_key() {
        let session_series = SessionSeries::gen();
        let key = format_key(&session_series);
        let expected = format!("{}{}{}", super::SESSION_ID_NAMESPACE, NAMESPACE_SEPARATOR, session_series);
        assert_eq!(key, expected);
    }
}