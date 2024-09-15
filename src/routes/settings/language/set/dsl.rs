use thiserror::Error;

use crate::common::{fallible::Fallible, id::AccountId, language::Language};

pub(crate) trait SetLanaguage {
    async fn set_language(&self, account_id: AccountId, language: Language) -> Fallible<(), SetLanguageError>;
}

#[derive(Debug, Error)]
pub enum SetLanguageError {
    #[error("言語の設定に失敗しました")]
    SetLanguageFailed(#[source] anyhow::Error),
}