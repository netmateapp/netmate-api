use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use super::dsl::{GetLanguage, GetLanguageError};

use crate::{common::{fallible::Fallible, id::AccountId, language::Language}, helper::{error::InitError, scylla::{prepare, Statement, TypedStatement}}};

pub struct GetLanguageImpl {
    session: Arc<Session>,
    select_language: SelectLanguage,
}

impl GetLanguageImpl {
    pub async fn try_new(session: Arc<Session>) -> Result<GetLanguageImpl, InitError<GetLanguageImpl>> {
        let select_language = prepare(&session, SelectLanguage, SELECT_LANGUAGE)
            .await
            .map_err(|e| InitError::new(e.into()))?;

        Ok(Self { session, select_language })
    }
}

impl GetLanguage for GetLanguageImpl {
    async fn get_language(&self, account_id: &AccountId) -> Fallible<Language, GetLanguageError> {
        self.select_language.execute(&self.session, (account_id, ))
            .await
            .map(|(language, )| language)
            .map_err(|e| GetLanguageError::GetLanguageFailed(e.into()))
    }
}

const SELECT_LANGUAGE: Statement<SelectLanguage> = Statement::of("SELECT language FROM accounts WHERE id = ? LIMIT 1");

struct SelectLanguage(Arc<PreparedStatement>);

impl<'a> TypedStatement<(&'a AccountId, ), (Language, )> for SelectLanguage {
    async fn execute(&self, session: &Arc<Session>, values: (&'a AccountId, )) -> anyhow::Result<(Language, )> {
        session.execute_unpaged(&self.0, values)
            .await
            .map_err(anyhow::Error::from)?
            .first_row_typed()
            .map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::{helper::scylla::check_cql_statement_type, routes::settings::language::get::interpreter::SELECT_LANGUAGE};
    
    #[test]
    fn check_select_language_type() {
        check_cql_statement_type(SELECT_LANGUAGE);
    }
}