use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{birth_year::BirthYear, email::address::Email, fallible::Fallible, id::account_id::AccountId, language::Language, one_time_token::OneTimeToken, password::PasswordHash, region::Region}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{VerifyEmail, VerifyEmailError};

pub struct VerifyEmailImpl {
    db: Arc<Session>,
    select_account_creation_application: Arc<PreparedStatement>,
    insert_account: Arc<PreparedStatement>,
    delete_account_creation_application: Arc<PreparedStatement>,
}

impl VerifyEmailImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<Self, InitError<VerifyEmailImpl>> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> InitError<VerifyEmailImpl> {
            InitError::new(e.into())
        }

        let select_account_creation_application = prepare(&db, "SELECT email, password_hash, birth_year, region, language FROM pre_verification_accounts WHERE one_time_token = ? LIMIT 1").await?;

        let insert_account = prepare(&db, "INSERT INTO accounts (id, email, password_hash, birth_year, region, language) VALUES (?, ?, ?, ?, ?, ?) IF NOT EXISTS").await?;

        let delete_account_creation_application = prepare(&db, "DELETE FROM pre_verification_accounts WHERE one_time_token = ?").await?;

        Ok(Self { db, select_account_creation_application, insert_account, delete_account_creation_application })
    }
}

impl VerifyEmail for VerifyEmailImpl {
    async fn retrieve_account_creation_application_by(&self, token: &OneTimeToken) -> Fallible<(Email, PasswordHash, BirthYear, Region, Language), VerifyEmailError> {
        self.db
            .execute_unpaged(&self.select_account_creation_application, (token, ))
            .await
            .map_err(|e| VerifyEmailError::RetrieveAccountCreationApplicationFailed(e.into()))?
            .first_row_typed()
            .map_err(|e| VerifyEmailError::RetrieveAccountCreationApplicationFailed(e.into()))
    }

    async fn create_account(&self, account_id: AccountId, email: &Email, password_hash: &PasswordHash, birth_year: BirthYear, region: Region, language: Language) -> Fallible<(), VerifyEmailError> {
        self.db
            .execute_unpaged(&self.insert_account, (account_id, email, password_hash, birth_year, region, language))
            .await
            .map(|_| ())
            .map_err(|e| VerifyEmailError::CreateAccountFailed(e.into()))
    }

    async fn delete_account_creation_application_by(&self, token: &OneTimeToken) -> Fallible<(), VerifyEmailError> {
        self.db
            .execute_unpaged(&self.delete_account_creation_application, (token, ))
            .await
            .map(|_| ())
            .map_err(|e| VerifyEmailError::DeleteAccountCreationApplicationFailed(e.into()))
    }
}