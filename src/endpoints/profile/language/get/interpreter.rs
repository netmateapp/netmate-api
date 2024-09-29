use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use super::dsl::{GetLanguage, GetLanguageError};

use crate::{common::{fallible::Fallible, profile::{account_id::AccountId, language::Language}}, helper::{error::InitError, scylla::prepare}};

pub struct GetLanguageImpl {
    db: Arc<Session>,
    select_language: Arc<PreparedStatement>,
}

impl GetLanguageImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<GetLanguageImpl, InitError<GetLanguageImpl>> {
        let select_language = prepare(&db, "SELECT language FROM accounts WHERE id = ?").await?;
        Ok(Self { db, select_language })
    }
}

impl GetLanguage for GetLanguageImpl {
    async fn get_language(&self, account_id: AccountId) -> Fallible<Language, GetLanguageError> {
        self.db
            .execute_unpaged(&self.select_language, (account_id, ))
            .await
            .map_err(|e| GetLanguageError::GetLanguageFailed(e.into()))?
            .first_row_typed::<(Language, )>()
            .map(|(language, )| language)
            .map_err(|e| GetLanguageError::GetLanguageFailed(e.into()))
    }
}