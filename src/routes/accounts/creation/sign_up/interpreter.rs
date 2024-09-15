use std::{str::FromStr, sync::{Arc, LazyLock}};

use scylla::{prepared_statement::PreparedStatement, FromRow, Session};

use crate::{common::{birth_year::BirthYear, email::{address::Email, resend::ResendEmailSender, send::{Body, EmailSender, HtmlContent, NetmateEmail, PlainText, SenderName, Subject}}, fallible::Fallible, id::account_id::AccountId, language::Language, password::PasswordHash, region::Region}, helper::{error::InitError, scylla::{Statement, TypedStatement, Unit}}, routes::accounts::creation::value::OneTimeToken, translation::{ja, us_en}};

use super::dsl::{SignUp, SignUpError};

pub struct SignUpImpl {
    db: Arc<Session>,
    select_account_id: Arc<SelectAccountId>,
    insert_account_creation_application: Arc<InsertAccountCreationApplication>,
}

impl SignUpImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<Self, InitError<SignUpImpl>> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> InitError<SignUpImpl> {
            InitError::new(e.into())
        }

        let select_account_id = SELECT_ACCOUNT_ID.prepared(&db, SelectAccountId)
            .await
            .map_err(handle_error)?;

        let insert_account_creation_application = INSERT_ACCOUNT_CREATION_APPLICATION.prepared(&db, InsertAccountCreationApplication)
            .await
            .map_err(handle_error)?;

        Ok(Self { db, select_account_id, insert_account_creation_application })
    }
}

static AUTHENTICATION_EMAIL_ADDRESS: LazyLock<NetmateEmail> = LazyLock::new(|| NetmateEmail::try_from(Email::from_str("verify-email@account.netmate.app").unwrap()).unwrap());
static JA_AUTHENTICATION_EMAIL_SUBJECT: LazyLock<Subject> = LazyLock::new(|| Subject::from_str(ja::sign_up::AUTHENTICATION_EMAIL_SUBJECT).unwrap());
static US_EN_AUTHENTICATION_EMAIL_SUBJECT: LazyLock<Subject> = LazyLock::new(|| Subject::from_str(us_en::sign_up::AUTHENTICATION_EMAIL_SUBJECT).unwrap());

impl SignUp for SignUpImpl {
    async fn is_available_email(&self, email: &Email) -> Fallible<bool, SignUpError> {
        self.select_account_id
            .query(&self.db, (email, ))
            .await
            .map(|v| v.is_some())
            .map_err(|e| SignUpError::PotentiallyUnavailableEmail(e.into()))
    }

    async fn apply_to_create_account(&self, email: &Email, pw_hash: &PasswordHash, birth_year: BirthYear, region: Region, language: Language, token: &OneTimeToken) -> Result<(), SignUpError> {
        self.insert_account_creation_application
            .execute(&self.db, (token, email, pw_hash, birth_year, region, language))
            .await
            .map(|_| ())
            .map_err(|e| SignUpError::ApplicationFailed(e.into()))
    }

    async fn send_verification_email(&self, email: &Email, language: Language, token: &OneTimeToken) -> Result<(), SignUpError> {
        let sender_name = SenderName::by(language);

        // ユーザーの設定言語に応じたテキストを取得する
        let (subject, html_content, plain_text) = match language {
            Language::Japanese => (&*JA_AUTHENTICATION_EMAIL_SUBJECT, ja::sign_up::ATUHENTICATION_EMAIL_BODY_HTML, ja::sign_up::AUTHENTICATION_EMAIL_BODY_PLAIN),
            _ => (&*US_EN_AUTHENTICATION_EMAIL_SUBJECT, us_en::sign_up::ATUHENTICATION_EMAIL_BODY_HTML, us_en::sign_up::AUTHENTICATION_EMAIL_BODY_PLAIN),
        };

        let body = Body::new(
            HtmlContent::new(&html_content.replace("{token}", token.value())),
            PlainText::new(&plain_text.replace("{token}", token.value()))
        );

        ResendEmailSender::send(&*AUTHENTICATION_EMAIL_ADDRESS, email, &sender_name, &subject, &body)
            .await
            .map_err(|e| SignUpError::AuthenticationEmailSendFailed(e.into()))
    }
}

const SELECT_ACCOUNT_ID: Statement<SelectAccountId> = Statement::of("SELECT id FROM accounts_by_email WHERE email = ? LIMIT 1");

struct SelectAccountId(PreparedStatement);

impl<'a> TypedStatement<(&'a Email, ), (AccountId, )> for SelectAccountId {
    type Result<U> = Option<U> where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (&'a Email, )) -> anyhow::Result<Self::Result<(AccountId, )>> {
        session.execute_unpaged(&self.0, values)
            .await
            .map_err(anyhow::Error::from)?
            .maybe_first_row_typed()
            .map_err(anyhow::Error::from)
    }
}

const INSERT_ACCOUNT_CREATION_APPLICATION: Statement<InsertAccountCreationApplication>
    = Statement::of("INSERT INTO account_creation_applications (ottoken, email, password_hash, birth_year, region, language) VALUES (?, ?, ?, ?, ?, ?) USING TTL 86400");

struct InsertAccountCreationApplication(PreparedStatement);

impl<'a, 'b, 'c> TypedStatement<(&'a OneTimeToken, &'b Email, &'c PasswordHash, BirthYear, Region, Language), Unit> for InsertAccountCreationApplication {
    type Result<U> = U where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (&'a OneTimeToken, &'b Email, &'c PasswordHash, BirthYear, Region, Language)) -> anyhow::Result<Unit> {
        session.execute_unpaged(&self.0, values)
            .await
            .map(|_| Unit)
            .map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::helper::scylla::{check_cql_query_type, check_cql_statement_type};

    use super::{INSERT_ACCOUNT_CREATION_APPLICATION, SELECT_ACCOUNT_ID};

    #[test]
    fn check_select_account_id_type() {
        check_cql_query_type(SELECT_ACCOUNT_ID);
    }

    #[test]
    fn check_insert_account_creation_application_type() {
        check_cql_statement_type(INSERT_ACCOUNT_CREATION_APPLICATION);
    }
}