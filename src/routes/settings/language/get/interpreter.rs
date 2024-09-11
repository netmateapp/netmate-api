use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use super::dsl::{GetLanguage, GetLanguageError};

use crate::{common::{fallible::Fallible, id::AccountId, language::Language}, cql, helper::{error::InitError, scylla::{prepare, TypedStatement}}};

pub struct GetLanguageImpl {
    session: Arc<Session>,
    select_language: UpdateLanguage,
}

impl GetLanguageImpl {
    pub async fn try_new(session: Arc<Session>) -> Result<GetLanguageImpl, InitError<GetLanguageImpl>> {
        let select_language = prepare::<InitError<GetLanguageImpl>>(
            &session,
            cql!("SELECT language FROM accounts WHERE id = ? LIMIT 1")
        )
        .await
        .map(UpdateLanguage)?;

        Ok(Self { session, select_language })
    }
}

impl GetLanguage for GetLanguageImpl {
    async fn get_language(&self, account_id: &AccountId) -> Fallible<Language, GetLanguageError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> GetLanguageError {
            GetLanguageError::GetLanguageFailed(e.into())
        }

        self.select_language.execute(&self.session, (account_id, ))
            .await
            .map(|(language, )| language)
            .map_err(handle_error)
    }
}

struct UpdateLanguage(Arc<PreparedStatement>);

impl<'a> TypedStatement<(&'a AccountId, ), (Language, )> for UpdateLanguage {
    async fn execute(&self, session: &Arc<Session>, values: (&'a AccountId, )) -> anyhow::Result<(Language, )> {
        session.execute_unpaged(&self.0, values)
            .await
            .map_err(anyhow::Error::from)?
            .first_row_typed()
            .map_err(anyhow::Error::from)
    }
}