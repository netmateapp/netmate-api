use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, FromRow, Session};

use super::dsl::{GetLanguage, GetLanguageError};

use crate::{common::{fallible::Fallible, id::AccountId, language::Language}, helper::{error::InitError, scylla::{prepare, Statement, TypedStatement}}};

pub struct GetLanguageImpl {
    db: Arc<Session>,
    select_language: SelectLanguage,
}

impl GetLanguageImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<GetLanguageImpl, InitError<GetLanguageImpl>> {
        let select_language = prepare(&db, SelectLanguage, SELECT_LANGUAGE)
            .await
            .map_err(|e| InitError::new(e.into()))?;

        Ok(Self { db, select_language })
    }
}

impl GetLanguage for GetLanguageImpl {
    async fn get_language(&self, account_id: &AccountId) -> Fallible<Language, GetLanguageError> {
        self.select_language.query(&self.db, (account_id, ))
            .await
            .map(|(language, )| language)
            .map_err(|e| GetLanguageError::GetLanguageFailed(e.into()))
    }
}

const SELECT_LANGUAGE: Statement<SelectLanguage> = Statement::of("SELECT language FROM accounts WHERE id = ? LIMIT 1");

struct SelectLanguage(Arc<PreparedStatement>);

impl<'a> TypedStatement<(&'a AccountId, ), (Language, )> for SelectLanguage {
    type Result<U> = U where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: (&'a AccountId, )) -> anyhow::Result<(Language, )> {
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