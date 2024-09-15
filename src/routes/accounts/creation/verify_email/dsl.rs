use thiserror::Error;

use crate::common::{birth_year::BirthYear, email::address::Email, fallible::Fallible, id::{account_id::AccountId, tag_id::TagId}, language::Language, one_time_token::OneTimeToken, password::PasswordHash, region::Region, tag::top_tag_id_by_language};

pub(crate) trait VerifyEmail {
    async fn verify_email(&self, token: &OneTimeToken) -> Fallible<(AccountId, TagId), VerifyEmailError> {
        let (email, password_hash, birth_year, region, language) = self.retrieve_account_creation_application_by(token).await?;
        let account_id = AccountId::gen();
        match self.create_account(account_id, &email, &password_hash, birth_year, region, language).await {
            Ok(_) => {
                // 失敗してもTTLにより削除されるため続行
                let _ = self.delete_account_creation_application_by(token).await;
                Ok((account_id, top_tag_id_by_language(&language)))
            },
            Err(VerifyEmailError::AccountAlreadyExists) => { // この状況は基本的に発生しない
                let _ = self.delete_account_creation_application_by(token).await;
                Err(VerifyEmailError::AccountAlreadyExists)
            },
            Err(e) => Err(e) 
        }
    }

    async fn retrieve_account_creation_application_by(&self, token: &OneTimeToken) -> Fallible<(Email, PasswordHash, BirthYear, Region, Language), VerifyEmailError>;

    async fn create_account(&self, account_id: AccountId, email: &Email, password_hash: &PasswordHash, birth_year: BirthYear, region: Region, language: Language) -> Fallible<(), VerifyEmailError>;

    async fn delete_account_creation_application_by(&self, token: &OneTimeToken) -> Fallible<(), VerifyEmailError>;
}

#[derive(Debug, Error)]
pub enum VerifyEmailError {
    #[error("アカウント作成申請データの取得に失敗しました")]
    RetrieveAccountCreationApplicationFailed(#[source] anyhow::Error),
    #[error("一時トークンによる認証に失敗しました")]
    OneTimeTokenAuthenticationFailed,
    #[error("アカウントの作成に失敗しました")]
    CreateAccountFailed(#[source] anyhow::Error),
    #[error("アカウントが既に存在しています")]
    AccountAlreadyExists,
    #[error("アカウント作成申請データの削除に失敗しました")]
    DeleteAccountCreationApplicationFailed(#[source] anyhow::Error),
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use thiserror::Error;

    use crate::common::{birth_year::BirthYear, email::address::Email, fallible::Fallible, id::{account_id::AccountId, tag_id::TagId}, language::Language, one_time_token::OneTimeToken, password::PasswordHash, region::Region};

    use super::{VerifyEmail, VerifyEmailError};

    struct MockVerifyEmail;

    #[derive(Debug, Error)]
    #[error("疑似エラー")]
    struct MockError;

    const RETRIEVE_FAILED: &str = "case1";
    const TOKEN_AUTH_FAILED: &str = "case2";
    const RETRIEVE_BUT_CREATE_FAILED: &str = "case3";
    const RETRIEVE_BUT_ACCOUNT_ALREADY_EXISTS: &str = "case4";
    const VERIFY_EMAIL: &str = "case5";

    impl VerifyEmail for MockVerifyEmail {
        async fn retrieve_account_creation_application_by(&self, case: &OneTimeToken) -> Fallible<(Email, PasswordHash, BirthYear, Region, Language), VerifyEmailError> {
            match case.value().as_str() {
                RETRIEVE_BUT_CREATE_FAILED | RETRIEVE_BUT_ACCOUNT_ALREADY_EXISTS | VERIFY_EMAIL => {
                    Ok((
                        Email::from_str("test@example.com").unwrap(),
                        PasswordHash::new_unchecked(case.value()),
                        BirthYear::try_from(0u16).unwrap(),
                        Region::Japan,
                        Language::Japanese
                    ))
                },
                TOKEN_AUTH_FAILED => Err(VerifyEmailError::OneTimeTokenAuthenticationFailed),
                RETRIEVE_FAILED => Err(VerifyEmailError::RetrieveAccountCreationApplicationFailed(MockError.into())),
                _ => panic!("予期しないエラーが発生しました")
            }
        }

        async fn create_account(&self, _: AccountId, _: &Email, case: &PasswordHash, _: BirthYear, _: Region, _: Language) -> Fallible<(), VerifyEmailError> {
            match case.value().as_str() {
                VERIFY_EMAIL => Ok(()),
                RETRIEVE_BUT_ACCOUNT_ALREADY_EXISTS => Err(VerifyEmailError::AccountAlreadyExists),
                RETRIEVE_BUT_CREATE_FAILED => Err(VerifyEmailError::CreateAccountFailed(MockError.into())),
                _ => panic!("予期しないエラーが発生しました")
            }
        }
    
        async fn delete_account_creation_application_by(&self, _: &OneTimeToken) -> Fallible<(), VerifyEmailError> {
            Ok(())
        }
    }

    async fn test_verify_email(case: &str) -> Fallible<(AccountId, TagId), VerifyEmailError> {
        MockVerifyEmail.verify_email(&OneTimeToken::new_unchecked(case)).await
    }

    #[tokio::test]
    async fn retrieve_failed() {
        match test_verify_email(RETRIEVE_FAILED).await.err().unwrap() {
            VerifyEmailError::RetrieveAccountCreationApplicationFailed(_) => (),
            _ => panic!("予期しないエラーが発生しました")
        }
    }

    #[tokio::test]
    async fn token_auth_failed() {
        match test_verify_email(TOKEN_AUTH_FAILED).await.err().unwrap() {
            VerifyEmailError::OneTimeTokenAuthenticationFailed => (),
            _ => panic!("予期しないエラーが発生しました")
        }
    }

    #[tokio::test]
    async fn retrieve_but_create_failed() {
        match test_verify_email(RETRIEVE_BUT_CREATE_FAILED).await.err().unwrap() {
            VerifyEmailError::CreateAccountFailed(_) => (),
            _ => panic!("予期しないエラーが発生しました")
        }
    }

    #[tokio::test]
    async fn retrieve_but_account_already_exists() {
        match test_verify_email(RETRIEVE_BUT_ACCOUNT_ALREADY_EXISTS).await.err().unwrap() {
            VerifyEmailError::AccountAlreadyExists => (),
            _ => panic!("予期しないエラーが発生しました")
        }
    }

    #[tokio::test]
    async fn verify_email() {
        assert!(test_verify_email(VERIFY_EMAIL).await.is_ok());
    }
}