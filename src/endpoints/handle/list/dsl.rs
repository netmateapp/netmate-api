use thiserror::Error;

use crate::common::{fallible::Fallible, handle::{id::HandleId, name::HandleName}, id::account_id::AccountId};
pub(crate) trait ListHandles {
    async fn list_handles(&self, account_id: AccountId) -> Fallible<Vec<(HandleId, Option<HandleName>)>, ListHandlesError>;
}

#[derive(Debug, Error)]
pub enum ListHandlesError {
    #[error("アカウントの名義の取得に失敗しました")]
    ListHandlesFailed(#[source] anyhow::Error),
}