use thiserror::Error;
use tracing::info;

use crate::common::{email::address::Email, fallible::Fallible, id::AccountId, language::Language};

pub(crate) trait MitigateSessionHijackingAttack {
    async fn mitigate(&self, account_id: &AccountId) {
        let is_email_sent = match self.fetch_email_and_language(&account_id).await {
            Ok((email, language)) => self.send_security_notification(&email, &language)
                .await
                .is_ok(),
            _ => false
        };

        let is_all_session_series_deleted = self.delete_all_session_series(&account_id).await.is_ok();

        info!(
            account_id = %account_id.value().value(),
            is_email_sent = is_email_sent,
            is_all_session_series_deleted = is_all_session_series_deleted,
            "セッション識別子の盗用の可能性を検出しました"
        );
    }

    async fn fetch_email_and_language(&self, account_id: &AccountId) -> Fallible<(Email, Language), MitigateSessionHijackingAttackError>;

    async fn send_security_notification(&self, email: &Email, language: &Language) -> Fallible<(), MitigateSessionHijackingAttackError>;

    async fn delete_all_session_series(&self, account_id: &AccountId) -> Fallible<(), MitigateSessionHijackingAttackError>;
}

#[derive(Debug, Error)]
pub enum MitigateSessionHijackingAttackError {
    #[error("メールアドレスと言語の取得に失敗しました")]
    FetchEmailAndLanguageFailed,
    #[error("セキュリティ通知に失敗しました")]
    SendSecurityNotificationFailed,
    #[error("全セッション系列の削除に失敗しました")]
    DeleteAllSessionSeriesFailed,
}