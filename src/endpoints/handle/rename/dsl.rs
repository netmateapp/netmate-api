use thiserror::Error;

use crate::common::{fallible::Fallible, handle::{id::HandleId, name::HandleName}, id::account_id::AccountId};

pub(crate) trait RenameHandle {
    async fn rename_handle_if_onymous(&self, account_id: AccountId, handle_id: HandleId, new_handle_name: HandleName) -> Fallible<(), RenameHandleError>;
}

#[derive(Debug, Error)]
pub enum RenameHandleError {
    #[error("名義の編集に失敗しました")]
    RenameHandleFailed(#[source] anyhow::Error)
}