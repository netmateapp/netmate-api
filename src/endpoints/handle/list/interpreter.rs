use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, handle::{id::HandleId, name::HandleName, share_count::HandleShareCount}, id::account_id::AccountId}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{GetHandles, GetHandlesError};

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

impl GetHandles for GetHandlesImpl {
    async fn get_handles(&self, account_id: AccountId) -> Fallible<Vec<(HandleId, HandleName, HandleShareCount)>, GetHandlesError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> GetHandlesError {
            GetHandlesError::GetHandlesFailed(e.into())
        }

        let handles = self.db
            .execute_unpaged(&self.select_handles, (account_id, ))
                .await
                .map_err(handle_error)?
                .rows_typed()
                .map(|rows| {
                    rows.flatten()
                        .collect::<Vec<(HandleId, HandleName)>>()
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