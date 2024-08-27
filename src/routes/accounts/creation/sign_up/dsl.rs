use thiserror::Error;

use crate::common::{birth_year::BirthYear, email::Email, language::Language, password::{Password, PasswordHash}, region::Region};

use super::value::OneTimeToken;

pub type Fallible<T, E> = Result<T, E>;

pub(crate) trait SignUp: Send {
    async fn sign_up(&self, email: &Email, password: &Password, birth_year: &BirthYear, region: &Region, language: &Language) -> Fallible<(), SignUpError> {
        if self.is_available_email(email).await? {
            // この位置でパスワードのハッシュ化が行われ高い負荷が発生するため、
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

#[derive(Debug, Error)]
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

#[cfg(test)]
mod tests {
    use std::{num::NonZeroU16, str::FromStr};

    use thiserror::Error;

    use crate::{common::{birth_year::BirthYear, email::Email, language::Language, password::{Password, PasswordHash}, region::Region}, routes::accounts::creation::sign_up::value::OneTimeToken};

    use super::{Fallible, SignUp, SignUpError};

    // `SignUp#sign_up()`のロジックの正当性を保証する
    struct MockSignUp;

    #[derive(Debug, Error)]
    #[error("疑似エラー")]
    struct MockError;

    const AVAILABLE: &str = "available@example.com";
    const UNAVAILABLE: &str = "unavailable@example.com";
    const POTENTIALLY_UNAVAILABLE: &str = "potentially_unavailable@example.com";
    const APPLY: &str = "apply@example.com";
    const SEND: &str = "send@example.com";

    impl SignUp for MockSignUp {
        async fn is_available_email(&self, email: &Email) -> Fallible<bool, SignUpError> {
            match email.value().as_str() {
                AVAILABLE | APPLY | SEND  => Ok(true),
                UNAVAILABLE => Ok(false),
                _ => Err(SignUpError::PotentiallyUnavailableEmail(MockError.into()))
            }
        }

        async fn apply_to_create_account(&self, email: &Email, _: &PasswordHash, _: &BirthYear, _: &Region, _: &Language, _: &OneTimeToken) -> Fallible<(), SignUpError> {
            match email.value().as_str() {
                APPLY | SEND => Ok(()),
                _ => Err(SignUpError::ApplicationFailed(MockError.into()))
            }
        }
    
        async fn send_verification_email(&self, email: &Email, _: &Language, _: &OneTimeToken) -> Result<(), SignUpError> {
            match email.value().as_str() {
                SEND => Ok(()),
                _ => Err(SignUpError::AuthenticationEmailSendFailed(MockError.into()))
            }
        }
    }

    async fn test_sign_up(email: &str) -> Result<(), SignUpError> {
        MockSignUp.sign_up(
            &Email::new_unchecked(email),
            &Password::from_str("vK,tOiHyLsehvnv").unwrap(),
            &BirthYear::new_unchecked(NonZeroU16::new(2000)),
            &Region::Japan,
            &Language::Japanese,
        ).await
    }

    #[tokio::test]
    async fn potentially_unavailable() {
        match test_sign_up(POTENTIALLY_UNAVAILABLE).await.err().unwrap() {
            SignUpError::PotentiallyUnavailableEmail(_) => (),
            _ => panic!("正しくないエラーが返されました")
        }
    }

    #[tokio::test]
    async fn unavailable() {
        match test_sign_up(UNAVAILABLE).await.err().unwrap() {
            SignUpError::UnavaialbleEmail => (),
            _ => panic!("正しくないエラーが返されました")
        }
    }

    #[tokio::test]
    async fn available() {
        match test_sign_up(AVAILABLE).await.err().unwrap() {
            SignUpError::ApplicationFailed(_) => (),
            _ => panic!("正しくないエラーが返されました")
        }
    }

    #[tokio::test]
    async fn apply() {
        match test_sign_up(APPLY).await.err().unwrap() {
            SignUpError::AuthenticationEmailSendFailed(_) => (),
            _ => panic!("正しくないエラーが返されました")
        }
    }

    #[tokio::test]
    async fn send() {
        assert!(test_sign_up(SEND).await.is_ok());
    }
}