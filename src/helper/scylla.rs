use std::{any::type_name, marker::PhantomData, sync::Arc};

use scylla::{cql_to_rust::FromRowError, frame::response::result::Row, prepared_statement::PreparedStatement, serialize::row::SerializeRow, transport::errors::QueryError, FromRow, Session};

use super::error::InitError;

#[macro_export]
macro_rules! cql {
    ($query:expr) => {
        $query
    };
}

pub async fn prep<T: From<QueryError>>(session: &Arc<Session>, query: &str) -> Result<Arc<PreparedStatement>, T> {
    match session.prepare(query).await {
        Ok(statement) => Ok(Arc::new(statement)),
        Err(e) => Err(T::from(e))
    }
}

impl<T> From<QueryError> for InitError<T> {
    fn from(value: QueryError) -> Self {
        Self::new(value.into())
    }
}

pub struct Statement<T>(&'static str, PhantomData<T>);

impl<T> Statement<T> {
    pub const fn of(statement: &'static str) -> Self {
        Self(statement, PhantomData)
    }
}

pub(crate) async fn prepare<I, O, T, C>(session: &Arc<Session>, constructor: C, statement: Statement<T>) -> Result<T, QueryError>
where
    I: SerializeRow,
    O: FromRow,
    T: TypedStatement<I, O>,
    C: FnOnce(Arc<PreparedStatement>) -> T
{
    match session.prepare(statement.0).await {
        Ok(statement) => Ok(constructor(Arc::new(statement))),
        Err(e) => Err(e)
    }
}

pub(crate) trait TypedStatement<I, O>
where
    I: SerializeRow,
    O: FromRow,
{
    type Result<U> where U: FromRow;

    async fn query(&self, db: &Arc<Session>, values: I) -> anyhow::Result<Self::Result<O>>;

    async fn execute(&self, db: &Arc<Session>, values: I) -> anyhow::Result<()> {
        self.query(db, values)
            .await
            .map(|_| ())
    }
}

// 孤児のルールにより`impl FromRow for ()`ができないため、`()`を代替する型として定義
pub struct Unit;

impl FromRow for Unit {
    fn from_row(_: Row) -> Result<Self, FromRowError> {
        Ok(Self)
    }
}

fn count_tuple_elements<T>() -> usize {
    let type_name = type_name::<T>();

    let comma_count = type_name.matches(',')
        .count();
    
    if type_name.starts_with('(') && type_name.ends_with(')') {
        if comma_count == 0 {
            0
        } else if type_name.ends_with(",)") {
            1
        } else {
            comma_count + 1
        }
    } else if type_name == "Unit" {
        0
    } else {
        panic!()
    }
}

// CQL文と`TypedStatement<I, O>`のパラメータと列の数がそれぞれ一致しているか確認する
// あくまで数の一致を確かめているだけであり、実際の列の型との比較は行っていない
pub(crate) fn check_cql_statement_type<I: SerializeRow, O: FromRow>(statement: Statement<impl TypedStatement<I, O>>) {
    let statement = statement.0;
    
    let value_count = statement.matches('?')
        .count();

    let column_count = &statement[(statement.find("SELECT").unwrap() + 6)..statement.find("FROM").unwrap()]
        .matches(',')
        .count() + 1;

    let values = count_tuple_elements::<I>();
    let columns = count_tuple_elements::<O>();

    assert_eq!(values, value_count);
    assert_eq!(columns, column_count);
}