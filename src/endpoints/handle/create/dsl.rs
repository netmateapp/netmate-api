use thiserror::Error;

use crate::common::{fallible::Fallible, handle::{id::HandleId, name::HandleName}, id::account_id::AccountId};

pub(crate) trait CreateHandle {
    async fn create_handle(&self, account_id: AccountId, handle_name: HandleName) -> Fallible<(), CreateHandleError> {
        self.create_new_handle(account_id, HandleId::gen(), handle_name).await
    }

    async fn create_new_handle(&self, account_id: AccountId, handle_id: HandleId, handle_name: HandleName) -> Fallible<(), CreateHandleError>;
}

#[derive(Debug, Error)]
pub enum CreateHandleError {
    #[error("名義の作成に失敗しました")]
    CreateHandleFailed(#[source] anyhow::Error)
}