use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, FromRow, Session};

use crate::{common::{fallible::Fallible, id::AccountId, language::Language}, helper::{error::InitError, scylla::{prepare, Statement, TypedStatement, Unit}}};

use super::dsl::{SetLanaguage, SetLanguageError};

pub struct SetLanguageImpl {
    db: Arc<Session>,
    update_language: Arc<UpdateLanguage>,
}

impl SetLanguageImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<SetLanguageImpl, InitError<SetLanguageImpl>> {
        let update_language = prepare(&db, UpdateLanguage, UPDATE_LANGUAGE)
            .await
            .map_err(|e| InitError::new(e.into()))?;

        Ok(Self { db, update_language })
    }
}

impl SetLanaguage for SetLanguageImpl {
    async fn set_language(&self, account_id: &AccountId, language: &Language) -> Fallible<(), SetLanguageError> {
        self.update_language
            .query(&self.db, (account_id, language))
            .await
            .map(|_| ()) // execute -ize
            .map_err(|e| SetLanguageError::SetLanguageFailed(e.into()))
    }
}

const UPDATE_LANGUAGE: Statement<UpdateLanguage>
    = Statement::of("UPDATE accounts SET language = ? WHERE id = ?");

struct UpdateLanguage(PreparedStatement);

impl<'a> TypedStatement<(&'a AccountId, &'a Language), Unit> for UpdateLanguage {
    type Result<U> = U where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (&'a AccountId, &'a Language)) -> anyhow::Result<Unit> {
        session.execute_unpaged(&self.0, values)
            .await
            .map(|_| Unit)
            .map_err(anyhow::Error::from)
    }
}