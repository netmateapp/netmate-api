use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, profile::{account_id::AccountId, language::Language}}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{SetLanaguage, SetLanguageError};

pub struct SetLanguageImpl {
    db: Arc<Session>,
    update_language: Arc<PreparedStatement>,
}

impl SetLanguageImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<SetLanguageImpl, InitError<SetLanguageImpl>> {
        let update_language = prepare(&db, "UPDATE accounts SET language = ? WHERE id = ?").await?;

        Ok(Self { db, update_language })
    }
}

impl SetLanaguage for SetLanguageImpl {
    async fn set_language(&self, account_id: AccountId, language: Language) -> Fallible<(), SetLanguageError> {
        self.db
            .execute_unpaged(&self.update_language, (language, account_id))
            .await
            .map(|_| ())
            .map_err(|e| SetLanguageError::SetLanguageFailed(e.into()))
    }
}