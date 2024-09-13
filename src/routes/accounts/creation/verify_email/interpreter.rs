use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, FromRow, Session};

use crate::{common::{birth_year::BirthYear, email::address::Email, fallible::Fallible, id::AccountId, language::Language, password::PasswordHash, region::Region}, helper::{error::InitError, scylla::{prepare, Statement, TypedStatement, Unit}}, routes::accounts::creation::value::OneTimeToken};

use super::dsl::{VerifyEmail, VerifyEmailError};

pub struct VerifyEmailImpl {
    db: Arc<Session>,
    select_account_creation_application: Arc<SelectAccountCreationApplication>,
    insert_account: Arc<InsertAccount>,
    delete_account_creation_application: Arc<DeleteAccountCreationApplication>,
}

impl VerifyEmailImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<Self, InitError<VerifyEmailImpl>> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> InitError<VerifyEmailImpl> {
            InitError::new(e.into())
        }

        let select_account_creation_application = prepare(&db, SelectAccountCreationApplication, SELECT_ACCOUNT_CREATION_APPLICATION)
            .await
            .map_err(handle_error)?;

        let insert_account = prepare(&db, InsertAccount, INSERT_ACCOUNT)
            .await
            .map_err(handle_error)?;

        let delete_account_creation_application = prepare(&db, DeleteAccountCreationApplication, DELETE_ACCOUNT_CREATION_APPLICATION)
            .await
            .map_err(handle_error)?;

        Ok(Self { db, select_account_creation_application, insert_account, delete_account_creation_application })
    }
}

impl VerifyEmail for VerifyEmailImpl {
    async fn retrieve_account_creation_application_by(&self, token: &OneTimeToken) -> Fallible<(Email, PasswordHash, BirthYear, Region, Language), VerifyEmailError> {
        self.select_account_creation_application
            .query(&self.db, (token, ))
            .await
            .map_err(|e| VerifyEmailError::RetrieveAccountCreationApplicationFailed(e.into()))
    }

    async fn create_account(&self, account_id: &AccountId, email: &Email, password_hash: &PasswordHash, birth_year: &BirthYear, region: &Region, language: &Language) -> Fallible<(), VerifyEmailError> {
        self.insert_account
            .query(&self.db, (account_id, email, password_hash, birth_year, region, language))
            .await
            .map(|_| ()) // execute -ize
            .map_err(|e| VerifyEmailError::CreateAccountFailed(e.into()))
    }

    async fn delete_account_creation_application_by(&self, token: &OneTimeToken) -> Fallible<(), VerifyEmailError> {
        self.delete_account_creation_application
            .query(&self.db, (token, ))
            .await
            .map(|_| ()) // execute -ize
            .map_err(|e| VerifyEmailError::DeleteAccountCreationApplicationFailed(e.into()))
    }
}

const SELECT_ACCOUNT_CREATION_APPLICATION: Statement<SelectAccountCreationApplication>
    = Statement::of("SELECT email, password_hash, birth_year, region, language FROM account_creation_applications WHERE ottoken = ? LIMIT 1");

struct SelectAccountCreationApplication(PreparedStatement);

impl<'a> TypedStatement<(&'a OneTimeToken, ), (Email, PasswordHash, BirthYear, Region, Language)> for SelectAccountCreationApplication {
    type Result<U> = U where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (&'a OneTimeToken, )) -> anyhow::Result<(Email, PasswordHash, BirthYear, Region, Language)> {
        session.execute_unpaged(&self.0, values)
            .await
            .map_err(anyhow::Error::from)?
            .first_row_typed()
            .map_err(anyhow::Error::from)
    }
}

const INSERT_ACCOUNT: Statement<InsertAccount>
    = Statement::of("INSERT INTO accounts (id, email, password_hash, birth_year, region, language) VALUES (?, ?, ?, ?, ?, ?) IF NOT EXISTS");

struct InsertAccount(PreparedStatement);

impl<'a, 'b, 'c, 'd, 'e, 'f> TypedStatement<(&'a AccountId, &'b Email, &'c PasswordHash, &'d BirthYear, &'e Region, &'f Language), Unit> for InsertAccount {
    type Result<U> = U where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (&'a AccountId, &'b Email, &'c PasswordHash, &'d BirthYear, &'e Region, &'f Language)) -> anyhow::Result<Unit> {
        session.execute_unpaged(&self.0, values)
            .await
            .map(|_| Unit)
            .map_err(anyhow::Error::from)
    }
}

const DELETE_ACCOUNT_CREATION_APPLICATION: Statement<DeleteAccountCreationApplication>
    = Statement::of("DELETE FROM account_creation_applications WHERE code = ?");

struct DeleteAccountCreationApplication(PreparedStatement);

impl<'a> TypedStatement<(&'a OneTimeToken, ), Unit> for DeleteAccountCreationApplication {
    type Result<U> = U where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (&'a OneTimeToken, )) -> anyhow::Result<Unit> {
        session.execute_unpaged(&self.0, values)
            .await
            .map(|_| Unit)
            .map_err(anyhow::Error::from)
    }
}