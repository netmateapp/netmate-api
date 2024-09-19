use thiserror::Error;

use crate::common::{fallible::Fallible, handle::id::HandleId, id::account_id::AccountId};

pub(crate) trait DeleteHandle {
    async fn delete_handle_if_onymous(&self, account_id: AccountId, handle_id: HandleId) -> Fallible<(), DeleteHandleError>;
}

#[derive(Debug, Error)]
pub enum DeleteHandleError {
    #[error("匿名名義は削除できません")]
    AnonymousHandle,
    #[error("名義の削除に失敗しました")]
    DeleteHandleFailed(#[source] anyhow::Error),
}