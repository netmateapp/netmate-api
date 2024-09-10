use std::{str::FromStr, sync::Arc};

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{birth_year::BirthYear, email::address::Email, fallible::Fallible, id::AccountId, language::Language, password::PasswordHash, region::Region}, helper::{error::InitError, scylla::prepare}, routes::accounts::creation::value::OneTimeToken};

use super::dsl::{VerifyEmail, VerifyEmailError};

pub struct VerifyEmailImpl {
    session: Arc<Session>,
    select_account_creation_application: Arc<PreparedStatement>,
    insert_account: Arc<PreparedStatement>,
    delete_account_creation_application: Arc<PreparedStatement>,
}

impl VerifyEmailImpl {
    pub async fn try_new(session: Arc<Session>) -> Result<Self, InitError<VerifyEmailImpl>> {
        let select_account_creation_application = prepare::<InitError<VerifyEmailImpl>>(
            &session,
            include_str!("select_account_creation_application.cql")
        ).await?;

        let insert_account = prepare::<InitError<VerifyEmailImpl>>(
            &session,
            include_str!("insert_account.cql")
        ).await?;

        let delete_account_creation_application = prepare::<InitError<VerifyEmailImpl>>(
            &session,
            include_str!("delete_account_creation_application.cql")
        ).await?;

        Ok(Self { session, select_account_creation_application, insert_account, delete_account_creation_application })
    }
}

impl VerifyEmail for VerifyEmailImpl {
    async fn retrieve_account_creation_application_by(&self, token: &OneTimeToken) -> Fallible<(Email, PasswordHash, BirthYear, Region, Language), VerifyEmailError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> VerifyEmailError {
            VerifyEmailError::RetrieveAccountCreationApplicationFailed(e.into())
        }

        let res = self.session
            .execute_unpaged(&self.select_account_creation_application, (token.value(), ))
            .await
            .map_err(handle_error)?;

        let (email, password_hash, birth_year, region, language) = res.first_row_typed::<(String, String, i16, i8, i8)>()
            .map_err(handle_error)?;

        let email = Email::from_str(email.as_str())
            .map_err(handle_error)?;
        let password_hash = PasswordHash::from_str(password_hash.as_str())
            .map_err(handle_error)?;
        let birth_year = BirthYear::try_from(birth_year)
            .map_err(handle_error)?;
        let region = Region::try_from(region)
            .map_err(handle_error)?;
        let language = Language::try_from(language)
            .map_err(handle_error)?;

        Ok((email, password_hash, birth_year, region, language))
    }

    async fn create_account(&self, account_id: &AccountId, email: &Email, password_hash: &PasswordHash, birth_year: &BirthYear, region: &Region, language: &Language) -> Fallible<(), VerifyEmailError> {
        let birth_year = i16::from(*birth_year);
        let region = i8::from(*region);
        let language = i8::from(*language);
        
        self.session
            .execute_unpaged(&self.insert_account, (account_id.value().value(), email.value(), password_hash.value(), birth_year, region, language))
            .await
            .map(|_| ())
            .map_err(|e| VerifyEmailError::CreateAccountFailed(e.into()))
    }

    async fn delete_account_creation_application_by(&self, token: &OneTimeToken) -> Fallible<(), VerifyEmailError> {
        self.session
            .execute_unpaged(&self.delete_account_creation_application, (token.value(), ))
            .await
            .map(|_| ())
            .map_err(|e| VerifyEmailError::DeleteAccountCreationApplicationFailed(e.into()))
    }
}
