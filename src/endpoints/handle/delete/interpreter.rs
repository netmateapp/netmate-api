use std::sync::Arc;

use scylla::{cql_to_rust::FromCqlVal, prepared_statement::PreparedStatement, Session};

use crate::{common::{fallible::Fallible, handle::id::HandleId, id::account_id::AccountId}, helper::{error::InitError, scylla::prepare}};

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
    async fn delete_handle_if_not_anonymous(&self, account_id: AccountId, handle_id: HandleId) -> Fallible<(), DeleteHandleError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> DeleteHandleError {
            DeleteHandleError::DeleteHandleFailed(e.into())
        }

        // 名義の削除を試行
        let result = self.db
            .execute_unpaged(&self.delete_handle_if_not_anonymous, (account_id, handle_id))
            .await
            .map_err(handle_error)?;

        let (applied_idx, _) = result.get_column_spec("applied")
            .ok_or_else(|| handle_error(anyhow::anyhow!("applied列がありません")))?;

        let applied = result.first_row()
            .map_err(handle_error)?
            .columns[applied_idx]
            .take();

        let applied = bool::from_cql(applied)
            .map_err(handle_error)?;

        if !applied {
            return Err(DeleteHandleError::AnonymousHandle);
        }

        // 共有数の削除
        self.db
            .execute_unpaged(&self.delete_handle_share_count, (account_id, handle_id))
            .await
            .map(|_| ())
            .map_err(handle_error)
    }
}