use thiserror::Error;

use crate::common::{birth_year::BirthYear, email::address::Email, fallible::Fallible, language::Language, one_time_token::OneTimeToken, password::{Password, PasswordHash}, region::Region};

pub(crate) trait SignUp {
    async fn sign_up(&self, email: &Email, password: &Password, birth_year: BirthYear, region: Region, language: Language) -> Fallible<(), SignUpError> {
        if self.is_available_email(email).await? {
            // この位置でパスワードのハッシュ化が行われ高い負荷が発生するため、
            // `sign_up`は自動化されたリクエストから特に保護されなければならない
            let hash: PasswordHash = password.hashed();
            let token = OneTimeToken::gen();
            self.apply_to_create_account(email, &hash, birth_year, region, language, &token).await?;
            self.send_verification_email(email, language, &token).await
        } else {
            Err(SignUpError::UnavailableEmail)
        }
    }

    async fn is_available_email(&self, email: &Email) -> Fallible<bool, SignUpError>;

    async fn apply_to_create_account(&self, email: &Email, pw_hash: &PasswordHash, birth_year: BirthYear, region: Region, language: Language, token: &OneTimeToken) -> Fallible<(), SignUpError>;

    async fn send_verification_email(&self, email: &Email, language: Language, token: &OneTimeToken) -> Result<(), SignUpError>;
}

#[derive(Debug, Error)]
pub enum SignUpError {
    #[error("指定のメールアドレスが利用可能である保証が得られませんでした")]
    PotentiallyUnavailableEmail(#[source] anyhow::Error),
    #[error("指定のメールアドレスは利用不能です")]
    UnavailableEmail,
    #[error("アカウント作成の申請に失敗しました")]
    ApplicationFailed(#[source] anyhow::Error),
    #[error("認証メールの送信に失敗しました")]
    AuthenticationEmailSendFailed(#[source] anyhow::Error)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use thiserror::Error;

    use crate::common::{birth_year::BirthYear, email::address::Email, language::Language, one_time_token::OneTimeToken, password::{Password, PasswordHash}, region::Region};

    use super::{Fallible, SignUp, SignUpError};

    // `SignUp#sign_up()`のロジックの正当性を保証する
    struct MockSignUp;

    #[derive(Debug, Error)]
    #[error("疑似エラー")]
    struct MockError;

    const POTENTIALLY_UNAVAILABLE: &str = "case1@example.com";
    const UNAVAILABLE: &str = "case2@example.com";
    const AVAILABLE_BUT_APPLICATION_FAILED: &str = "case3@example.com";
    const APPLIED_BUT_SEND_FAILED: &str = "case4@example.com";
    const SIGN_UP: &str = "case5@example.com";

    impl SignUp for MockSignUp {
        async fn is_available_email(&self, case: &Email) -> Fallible<bool, SignUpError> {
            match case.value().as_str() {
                AVAILABLE_BUT_APPLICATION_FAILED | APPLIED_BUT_SEND_FAILED | SIGN_UP  => Ok(true),
                UNAVAILABLE => Ok(false),
                _ => Err(SignUpError::PotentiallyUnavailableEmail(MockError.into()))
            }
        }

        async fn apply_to_create_account(&self, case: &Email, _: &PasswordHash, _: BirthYear, _: Region, _: Language, _: &OneTimeToken) -> Fallible<(), SignUpError> {
            match case.value().as_str() {
                APPLIED_BUT_SEND_FAILED | SIGN_UP => Ok(()),
                _ => Err(SignUpError::ApplicationFailed(MockError.into()))
            }
        }
    
        async fn send_verification_email(&self, case: &Email, _: Language, _: &OneTimeToken) -> Fallible<(), SignUpError> {
            match case.value().as_str() {
                SIGN_UP => Ok(()),
                _ => Err(SignUpError::AuthenticationEmailSendFailed(MockError.into()))
            }
        }
    }

    async fn test_sign_up(case: &str) -> Fallible<(), SignUpError> {
        MockSignUp.sign_up(
            &Email::from_str(case).unwrap(),
            &Password::from_str("vK,tOiHyLsehvnv").unwrap(),
            BirthYear::try_from(2000u16).unwrap(),
            Region::Japan,
            Language::Japanese,
        ).await
    }

    #[tokio::test]
    async fn potentially_unavailable() {
        match test_sign_up(POTENTIALLY_UNAVAILABLE).await.err().unwrap() {
            SignUpError::PotentiallyUnavailableEmail(_) => (),
            _ => panic!("予期しないエラーが発生しました")
        }
    }

    #[tokio::test]
    async fn unavailable() {
        match test_sign_up(UNAVAILABLE).await.err().unwrap() {
            SignUpError::UnavailableEmail => (),
            _ => panic!("予期しないエラーが発生しました")
        }
    }

    #[tokio::test]
    async fn available_but_application_failed() {
        match test_sign_up(AVAILABLE_BUT_APPLICATION_FAILED).await.err().unwrap() {
            SignUpError::ApplicationFailed(_) => (),
            _ => panic!("予期しないエラーが発生しました")
        }
    }

    #[tokio::test]
    async fn applied_but_send_failed() {
        match test_sign_up(APPLIED_BUT_SEND_FAILED).await.err().unwrap() {
            SignUpError::AuthenticationEmailSendFailed(_) => (),
            _ => panic!("予期しないエラーが発生しました")
        }
    }

    #[tokio::test]
    async fn sign_up() {
        assert!(test_sign_up(SIGN_UP).await.is_ok());
    }
}