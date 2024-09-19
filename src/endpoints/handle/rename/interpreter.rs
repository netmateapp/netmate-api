use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, handle::{id::HandleId, name::HandleName}, id::account_id::AccountId}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{RenameHandle, RenameHandleError};

pub struct RenameHandleImpl {
    db: Arc<Session>,
    update_handle_name: Arc<PreparedStatement>,
}

impl RenameHandleImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<Self, InitError<Self>> {
        let update_handle_name = prepare(&db, "UPDATE handles SET handle_name = ? WHERE account_id = ? AND handle_id = ?").await?;

        Ok(Self { db, update_handle_name })
    }
}

impl RenameHandle for RenameHandleImpl {
    async fn rename_handle(&self, account_id: AccountId, handle_id: HandleId, new_handle_name: HandleName) -> Fallible<(), RenameHandleError> {
        self.db
            .execute_unpaged(&self.update_handle_name, (new_handle_name, account_id, handle_id))
            .await
            .map(|_| ())
            .map_err(|e| RenameHandleError::RenameHandleFailed(e.into()))
    }
}