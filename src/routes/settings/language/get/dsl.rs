use thiserror::Error;

use crate::common::{fallible::Fallible, id::AccountId, language::Language};

pub(crate) trait GetLanguage {
    async fn get_language(&self, account_id: AccountId) -> Fallible<Language, GetLanguageError>;
}

#[derive(Debug, Error)]
#[error("言語設定の取得に失敗しました")]
pub enum GetLanguageError {
    GetLanguageFailed(#[source] anyhow::Error),
}