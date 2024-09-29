use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, handle::{id::HandleId, name::HandleName}, profile::account_id::AccountId}, helper::{error::InitError, scylla::prepare}};

use super::dsl::{CreateHandle, CreateHandleError};

pub struct CreateHandleImpl {
    db: Arc<Session>,
    insert_handle: Arc<PreparedStatement>,
    insert_handle_share_count: Arc<PreparedStatement>,
}

impl CreateHandleImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<Self, InitError<Self>> {
        let insert_handle = prepare(&db, "INSERT INTO handles (account_id, handle_id, handle_name) VALUES (?, ?, ?)").await?;
        
        let insert_handle_share_count = prepare(&db, "INSERT INTO handle_share_counts (account_id, handle_id, share_count) VALUES (?, ?, ?)").await?;

        Ok(Self { db, insert_handle, insert_handle_share_count })
    }
}

impl CreateHandle for CreateHandleImpl {
    async fn add_handle(&self, account_id: AccountId, handle_id: HandleId, handle_name: HandleName) -> Fallible<(), CreateHandleError> {
        self.db
            .execute_unpaged(&self.insert_handle, (account_id, handle_id, handle_name))
            .await
            .map(|_| ())
            .map_err(|e| CreateHandleError::CreateHandleFailed(e.into()))?;

        self.db
            .execute_unpaged(&self.insert_handle_share_count, (account_id, handle_id, 0))
            .await
            .map(|_| ())
            .map_err(|e| CreateHandleError::CreateHandleFailed(e.into()))
    }
}