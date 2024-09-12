use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use super::dsl::{GetLanguage, GetLanguageError};

use crate::{common::{fallible::Fallible, id::AccountId, language::Language}, cql, helper::{error::InitError, scylla::{prep, prepare, Statement, TypedStatement}}};

pub struct GetLanguageImpl {
    session: Arc<Session>,
    select_language: SelectLanguage,
}

// const NAME = cql!(SelectLanguage, "SELECT language FROM accounts WHERE id = ? LIMIT 1");
const SELECT_LANGUAGE: Statement<SelectLanguage> = Statement::of("SELECT language FROM accounts WHERE id = ? LIMIT 1");

impl GetLanguageImpl {
    pub async fn try_new(session: Arc<Session>) -> Result<GetLanguageImpl, InitError<GetLanguageImpl>> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> InitError<GetLanguageImpl> {
            InitError::new(e.into())
        }

        let select_language = prepare(&session, SelectLanguage, SELECT_LANGUAGE)
            .await
            .map_err(handle_error)?;

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
    use std::{any::type_name, marker::PhantomData};


    use scylla::{serialize::row::SerializeRow, FromRow};

    use crate::{helper::scylla::TypedStatement, routes::settings::language::get::interpreter::SelectLanguage};

    fn number_of_values_and_result_cols<I: SerializeRow, O: FromRow>(_: PhantomData<impl TypedStatement<I, O>>) -> (usize, usize) {
        let values = count_tuple_elements::<I>();
        let result_cols = count_tuple_elements::<O>();
        (values, result_cols)
    }

    fn count_tuple_elements<T>() -> usize {
        let type_name = type_name::<T>();
        println!("{}", type_name);

        // カンマの数を数える
        let comma_count = type_name.matches(',').count();
        
        // タプルの要素数は、カンマの数 + 1
        // ただし、単一要素のタプルの場合でもカンマがないため特別に処理する
        if type_name.starts_with('(') && type_name.ends_with(')') {
            if comma_count == 0 {
                0 // 単一要素のタプル
            } else if type_name.ends_with(",)") {
                1
            } else {
                comma_count + 1
            }
        } else {
            panic!() // タプルではない
        }
    }

    #[test]
    fn validate_cql() {
        let stmt = "SELECT language FROM accounts WHERE id = ? LIMIT 1";
        let params = stmt.matches('?').count();
        let results = &stmt[(stmt.find("SELECT").unwrap() + 6)..stmt.find("FROM").unwrap()];
        let results = results.matches(',').count() + 1;

        let (values, result_cols) = number_of_values_and_result_cols(PhantomData::<SelectLanguage>);
        assert_eq!(values, params);
        assert_eq!(result_cols, results);
    }
}