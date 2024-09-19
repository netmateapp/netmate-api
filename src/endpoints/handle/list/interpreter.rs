use std::{str::FromStr, sync::Arc};

use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::CqlValue, prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, handle::{id::HandleId, name::HandleName, share_count::HandleShareCount}, id::account_id::AccountId}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{ListHandles, GetHandlesError};

pub struct GetHandlesImpl {
    db: Arc<Session>,
    select_handles: Arc<PreparedStatement>,
    select_handle_share_counts: Arc<PreparedStatement>,
}

impl GetHandlesImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<Self, InitError<Self>> {
        let select_handles = prepare(&db, "SELECT handle_id, handle_name FROM handles WHERE account_id = ?").await?;

        let select_handle_share_counts = prepare(&db, "SELECT share_count FROM handle_share_counts WHERE account_id = ?").await?;

        Ok(Self { db, select_handles, select_handle_share_counts })
    }
}

impl ListHandles for GetHandlesImpl {
    async fn list_handles(&self, account_id: AccountId) -> Fallible<Vec<(HandleId, Option<HandleName>, HandleShareCount)>, GetHandlesError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> GetHandlesError {
            GetHandlesError::GetHandlesFailed(e.into())
        }

        let handles: Vec<(HandleId, Option<HandleName>)> = self.db
            .execute_unpaged(&self.select_handles, (account_id, ))
                .await
                .map_err(handle_error)?
                .rows_typed()
                .map(|rows| {
                    rows.flatten()
                        .map(|(id, name): (HandleId, OptionHandleName)| (id, name.0))
                        .collect::<Vec<(HandleId, Option<HandleName>)>>()
                })
                .map_err(handle_error)?;

        let handle_share_counts = self.db
            .execute_unpaged(&self.select_handle_share_counts, (account_id, ))
            .await
            .map_err(handle_error)?
            .rows_typed()
            .map(|rows| {
                rows.flatten()
                    .collect::<Vec<(HandleShareCount, )>>()
            })
            .map_err(handle_error)?;

        let handles = handles.into_iter()
            .zip(handle_share_counts.into_iter())
            .map(|((handle_id, handle_name), (handle_share_count, ))| (handle_id, handle_name, handle_share_count))
            .collect();

        Ok(handles)
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