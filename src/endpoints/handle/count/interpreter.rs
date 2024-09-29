use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, handle::{id::HandleId, share_count::HandleShareCount}, profile::account_id::AccountId}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{CountHandlesShare, CountHandlesShareError};

pub struct CountHandlesShareImpl {
    db: Arc<Session>,
    select_handle_share_counts: Arc<PreparedStatement>,
}

impl CountHandlesShareImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<Self, InitError<Self>> {
        let select_handle_share_counts = prepare(&db, "SELECT handle_id, share_count FROM handle_share_counts WHERE account_id = ?").await?;

        Ok(Self { db, select_handle_share_counts })
    }
}

impl CountHandlesShare for CountHandlesShareImpl {
    async fn count_handles_share(&self, account_id: AccountId) -> Fallible<Vec<(HandleId, HandleShareCount)>, CountHandlesShareError> {
        self.db
            .execute_unpaged(&self.select_handle_share_counts, (account_id, ))
            .await
            .map_err(|e| CountHandlesShareError::CountHandlesShareFailed(e.into()))?
            .rows_typed()
            .map(|rows| {
                rows.flatten()
                    .collect::<Vec<(HandleId, HandleShareCount)>>()
            })
            .map_err(|e| CountHandlesShareError::CountHandlesShareFailed(e.into()))
    }
}