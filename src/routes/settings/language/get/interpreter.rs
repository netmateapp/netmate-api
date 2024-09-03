use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use super::dsl::{GetLanguage, GetLanguageError};

use crate::{common::{fallible::Fallible, id::AccountId, language::Language}, helper::{error::{DslErrorMapper, InitError}, scylla::prepare}};

pub struct GetLanguageImpl {
    session: Arc<Session>,
    select_language: Arc<PreparedStatement>,
}

impl GetLanguageImpl {
    pub async fn try_new(session: Arc<Session>) -> Result<GetLanguageImpl, InitError<GetLanguageImpl>> {
        let select_language = prepare::<InitError<GetLanguageImpl>>(
            &session,
            "SELECT language FROM accounts WHERE id = ?"
        ).await?;

        Ok(Self { session, select_language })
    }
}

impl GetLanguage for GetLanguageImpl {
    async fn get_language(&self, account_id: &AccountId) -> Fallible<Language, GetLanguageError> {
        let language: Language = self.session
            .execute(&self.select_language, (account_id.value(), ))
            .await
            .map_dsl_error()?
            .first_row_typed::<(i8, )>()
            .map_dsl_error()?
            .0
            .try_into()
            .map_dsl_error()?;
        
        Ok(language)
    }
}

impl <T, U: Into<anyhow::Error>> DslErrorMapper<T, GetLanguageError> for Result<T, U> {
    fn map_dsl_error(self) -> Result<T, GetLanguageError> {
        self.map_err(|e| GetLanguageError::GetLanguageFailed(e.into()))
    }
}