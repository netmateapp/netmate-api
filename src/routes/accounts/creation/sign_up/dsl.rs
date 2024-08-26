use crate::{common::{birth_year::BirthYear, email::Email, language::Language, password::{Password, PasswordHash}, region::Region, send_email::{Body, NetmateEmail, ResendEmailService, SenderNameLocale, Subject, TransactionalEmailService}}, translation::{ja, us_en}};

use super::value::OneTimeToken;

pub type Fallible<T, E> = Result<T, E>;

pub(crate) trait SignUp: Send {
    async fn sign_up(&self, email: &Email, password: &Password, birth_year: &BirthYear, region: &Region, language: &Language) -> Fallible<(), SignUpError> {
        if self.is_available_email(email).await? {
            // この位置でパスワードのハッシュ化を行う必要があり高い負荷が発生するため、
            // `sign_up`は自動化されたリクエストから特に保護されなければならない
            let hash: PasswordHash = password.hashed();
            let token = OneTimeToken::generate();
            self.apply_to_create_account(email, &hash, birth_year, region, language, &token).await?;
            self.send_verification_email(email, language, &token).await
        } else {
            Err(SignUpError::UnavaialbleEmail)
        }
    }

    async fn is_available_email(&self, email: &Email) -> Fallible<bool, SignUpError>;

    async fn apply_to_create_account(&self, email: &Email, pw_hash: &PasswordHash, birth_year: &BirthYear, region: &Region, language: &Language, token: &OneTimeToken) -> Fallible<(), SignUpError>;

    async fn send_verification_email(&self, email: &Email, language: &Language, token: &OneTimeToken) -> Result<(), SignUpError>;
}

#[derive(Debug, thiserror::Error)]
pub enum SignUpError {
    #[error("指定のメールアドレスが利用可能である保証が得られませんでした")]
    PotentiallyUnavailableEmail(#[source] anyhow::Error),
    #[error("指定のメールアドレスは利用不能です")]
    UnavaialbleEmail,
    #[error("アカウント作成の申請に失敗しました")]
    ApplicationFailed(#[source] anyhow::Error),
    #[error("認証メールの送信に失敗しました")]
    AuthenticationEmailSendFailed(#[source] anyhow::Error)
}