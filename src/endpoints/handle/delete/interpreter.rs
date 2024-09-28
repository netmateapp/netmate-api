use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, handle::id::HandleId, id::account_id::AccountId}, helper::{error::InitError, scylla::{prepare, Transactional}}};

use super::dsl::{DeleteHandle, DeleteHandleError};

pub struct DeleteHandleImpl {
    db: Arc<Session>,
    delete_handle_if_not_anonymous: Arc<PreparedStatement>,
    delete_handle_share_count: Arc<PreparedStatement>,
}

impl DeleteHandleImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<Self, InitError<Self>> {
        let delete_handle_if_not_anonymous = prepare(&db, "DELETE FROM handles WHERE account_id = ? AND handle_id = ? IF handle_name != ''").await?;

        let delete_handle_share_count = prepare(&db, "DELETE FROM handle_share_counts WHERE account_id = ? AND handle_id = ?").await?;

        Ok(Self { db, delete_handle_if_not_anonymous, delete_handle_share_count })
    }
}

impl DeleteHandle for DeleteHandleImpl {
    async fn delete_handle_if_onymous(&self, account_id: AccountId, handle_id: HandleId) -> Fallible<(), DeleteHandleError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> DeleteHandleError {
            DeleteHandleError::DeleteHandleFailed(e.into())
        }

        // 名義の削除を試行
        self.db
            .execute_unpaged(&self.delete_handle_if_not_anonymous, (account_id, handle_id))
            .await
            .applied(DeleteHandleError::DeleteHandleFailed, || DeleteHandleError::AnonymousHandle)?;

        // 共有数の削除
        self.db
            .execute_unpaged(&self.delete_handle_share_count, (account_id, handle_id))
            .await
            .map(|_| ())
            .map_err(handle_error)
    }
}