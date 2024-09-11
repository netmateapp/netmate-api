use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, id::AccountId, language::Language}, cql, helper::{error::InitError, scylla::prepare}};

use super::dsl::{SetLanaguage, SetLanguageError};

pub struct SetLanguageImpl {
    db: Arc<Session>,
    update_language: Arc<PreparedStatement>,
}

impl SetLanguageImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<SetLanguageImpl, InitError<SetLanguageImpl>> {
        let update_language = prepare::<InitError<SetLanguageImpl>>(
            &db,
            cql!("UPDATE accounts SET language = ? WHERE id = ?")
        ).await?;

        Ok(Self { db, update_language })
    }
}

impl SetLanaguage for SetLanguageImpl {
    async fn set_language(&self, account_id: &AccountId, language: &Language) -> Fallible<(), SetLanguageError> {
        let values = (i8::from(*language), account_id.to_string());

        self.db
            .execute_unpaged(&self.update_language, values)
            .await
            .map(|_| ())
            .map_err(|e| SetLanguageError::SetLanguageFailed(e.into()))
    }
}