use std::{str::FromStr, sync::Arc};

use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::CqlValue, prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, handle::{id::HandleId, name::HandleName}, id::account_id::AccountId}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{ListHandles, ListHandlesError};

pub struct ListHandlesImpl {
    db: Arc<Session>,
    select_handles: Arc<PreparedStatement>,
}

impl ListHandlesImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<Self, InitError<Self>> {
        let select_handles = prepare(&db, "SELECT handle_id, handle_name FROM handles WHERE account_id = ?").await?;

        Ok(Self { db, select_handles })
    }
}

impl ListHandles for ListHandlesImpl {
    async fn list_handles(&self, account_id: AccountId) -> Fallible<Vec<(HandleId, Option<HandleName>)>, ListHandlesError> {
        self.db
            .execute_unpaged(&self.select_handles, (account_id, ))
                .await
                .map_err(|e| ListHandlesError::ListHandlesFailed(e.into()))?
                .rows_typed()
                .map(|rows| {
                    rows.flatten()
                        .map(|(id, name): (HandleId, OptionHandleName)| (id, name.0))
                        .collect::<Vec<(HandleId, Option<HandleName>)>>()
                })
                .map_err(|e| ListHandlesError::ListHandlesFailed(e.into()))
    }
}

pub struct OptionHandleName(Option<HandleName>);

impl FromCqlVal<Option<CqlValue>> for OptionHandleName {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        String::from_cql(cql_val)
            .and_then(|v| {
                if v.is_empty() {
                    Ok(OptionHandleName(None))
                } else {
                    HandleName::from_str(&v)
                        .map(|v| OptionHandleName(Some(v)))
                        .map_err(|_| FromCqlValError::BadVal)
                }
            })
    }
}