use thiserror::Error;

use crate::common::{fallible::Fallible, handle::name::HandleName, id::account_id::AccountId};

pub(crate) trait CreateHandle {
    async fn create_handle(&self, account_id: AccountId, new_handle_name: HandleName) -> Fallible<(), CreateHandleError>;
}

#[derive(Debug, Error)]
pub enum CreateHandleError {

}