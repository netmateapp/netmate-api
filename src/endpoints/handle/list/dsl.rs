use thiserror::Error;

use crate::common::{fallible::Fallible, handle::{id::HandleId, name::HandleName, share_count::HandleShareCount}, id::account_id::AccountId};
pub(crate) trait ListHandles {
    async fn list_handles(&self, account_id: AccountId) -> Fallible<Vec<(HandleId, Option<HandleName>, HandleShareCount)>, GetHandlesError>;
}

#[derive(Debug, Error)]
pub enum GetHandlesError {
    #[error("アカウントの名義の取得に失敗しました")]
    GetHandlesFailed(#[source] anyhow::Error),
}