use std::{any::type_name, collections::HashSet, marker::PhantomData, sync::{Arc, LazyLock}};

use regex::Regex;
use scylla::{cql_to_rust::FromRowError, frame::response::result::Row, prepared_statement::PreparedStatement, serialize::row::SerializeRow, transport::errors::QueryError, FromRow, Session};

use crate::common::db::{Column, Table};

use super::error::InitError;

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

    pub(crate) async fn prepared<I, O, S, C>(&self, session: &Arc<Session>, constructor: C) -> Result<Arc<S>, QueryError>
    where
        I: SerializeRow,
        O: FromRow,
        S: TypedStatement<I, O>,
        C: FnOnce(PreparedStatement) -> S
    {
        match session.prepare(self.0).await {
            Ok(statement) => Ok(Arc::new(constructor(statement))),
            Err(e) => Err(e)
        }
    }
}

pub(crate) trait TypedStatement<I, O>
where
    I: SerializeRow,
    O: FromRow,
{
    type Result<U> where U: FromRow;

    async fn query(&self, session: &Arc<Session>, values: I) -> anyhow::Result<Self::Result<O>>;

    async fn execute(&self, session: &Arc<Session>, values: I) -> anyhow::Result<()> {
        self.query(session, values)
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
    } else {
        panic!()
    }
}

// CQL文の正当性は検証するが、設計の誤り(e.g. パーティションキーの指定漏れ等)は検証しない

// CQL文と`TypedStatement<I, O>`のパラメータと列の数がそれぞれ一致しているか確認する
// あくまで数の一致を確かめているだけであり、実際の列の型との比較は行っていない
#[allow(unused)]
pub(crate) fn check_cql_query_type<I: SerializeRow, O: FromRow>(statement: Statement<impl TypedStatement<I, O>>) {
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

static SELECT_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"SELECT\s+(?P<columns>[a-zA-Z0-9_,\s]+)\s+FROM\s+(?P<table>[a-zA-Z0-9_]+)\s+WHERE\s+(?P<keys>[a-zA-Z0-9_\s=?.AND]+?)(?:\s+LIMIT|\s+USING|$)").unwrap());

pub(crate) fn check_cql_query_typed<I: SerializeRow, O: FromRow, T>(statement: Statement<impl TypedStatement<I, O>>, tb: Table<T>, selector_columns: &[Column<T>], selected_columns: &[Column<T>]) {
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

    if let Some(caps) = SELECT_REGEX.captures(statement) {
        let mut columns: HashSet<String> = caps.name("columns")
            .unwrap()
            .as_str()
            .trim()
            .split(", ")
            .map(|s| s.to_string())
            .collect();

        for col in selected_columns {
            assert!(columns.remove(col.name()), "{} 列はクエリで選択されていません", col.name());
        }

        let table = caps.name("table").unwrap().as_str();
        assert_eq!(table, tb.name());

        let mut keys = caps.name("keys")
            .unwrap()
            .as_str()
            .trim()
            .to_string();

        keys.push_str(" AND ");

        let mut keys: HashSet<String> = keys.split(" = ? AND ")
            .map(|s| s.to_string())
            .collect();

        for key in selector_columns {
            assert!(keys.remove(key.name()), "{} 列はクエリのキーに使用されていません", key.name());
        }
    } else {
        panic!()
    }
}

#[allow(unused)]
pub(crate) fn check_cql_statement_type<I: SerializeRow>(statement: Statement<impl TypedStatement<I, Unit>>) {
    let value_count = statement.0.matches('?')
        .count();

    let values = count_tuple_elements::<I>();

    assert_eq!(values, value_count);
}