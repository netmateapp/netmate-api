use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use super::dsl::{GetLanguage, GetLanguageError};

use crate::{common::{fallible::Fallible, id::AccountId, language::Language}, helper::{error::InitError, scylla::prepare}};

pub struct GetLanguageImpl {
    session: Arc<Session>,
    select_language: Arc<PreparedStatement>,
}

impl GetLanguageImpl {
    pub async fn try_new(session: Arc<Session>) -> Result<GetLanguageImpl, InitError<GetLanguageImpl>> {
        let select_language = prepare::<InitError<GetLanguageImpl>>(
            &session,
            include_str!("select_language.cql")
        ).await?;

        Ok(Self { session, select_language })
    }
}

impl GetLanguage for GetLanguageImpl {
    async fn get_language(&self, account_id: &AccountId) -> Fallible<Language, GetLanguageError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> GetLanguageError {
            GetLanguageError::GetLanguageFailed(e.into())
        }

        self.session
            .execute_unpaged(&self.select_language, (account_id.value().value(), ))
            .await
            .map_err(handle_error)?
            .first_row_typed::<(i8, )>()
            .map_err(handle_error)?
            .0
            .try_into()
            .map_err(handle_error)
    }
}