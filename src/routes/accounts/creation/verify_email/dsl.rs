use thiserror::Error;
use uuid::Uuid;

use crate::{common::{birth_year::BirthYear, email::Email, fallible::Fallible, language::Language, password::PasswordHash, region::Region}, routes::accounts::creation::sign_up::value::OneTimeToken};

pub(crate) trait VerifyEmail {
    async fn verify_email(&self, token: &OneTimeToken) -> Fallible<(), VerifyEmailError> {
        let (email, password_hash, birth_year, region, language) = self.retrieve_account_creation_application_by(token).await?;
        let account_id: AccountId = Uuid7::new_unchecked(Uuid::now_v7());
        match self.create_account(&account_id, &email, &password_hash, &birth_year, &region, &language).await {
            Ok(_) => {
                // 失敗してもTTLにより削除されるため続行
                let _ = self.delete_account_creation_application_by(token).await;
                Ok(())
            },
            Err(VerifyEmailError::AccountAlreadyExists) => { // この状況は基本的に発生しない
                let _ = self.delete_account_creation_application_by(token).await;
                Err(VerifyEmailError::AccountAlreadyExists)
            },
            Err(e) => Err(e) 
        }
    }

    async fn retrieve_account_creation_application_by(&self, token: &OneTimeToken) -> Fallible<(Email, PasswordHash, BirthYear, Region, Language), VerifyEmailError>;

    async fn create_account(&self, account_id: &AccountId, email: &Email, password_hash: &PasswordHash, birth_year: &BirthYear, region: &Region, language: &Language) -> Fallible<(), VerifyEmailError>;

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

pub type AccountId = Uuid7;

pub struct Uuid7(Uuid);

impl Uuid7 {
    pub fn new_unchecked(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn value(&self) -> &Uuid {
        &self.0
    }
}

#[derive(Debug, Error)]
#[error("UUIDのバージョンが7ではありません")]
pub struct ParseUuid7Error;

impl TryFrom<Uuid> for Uuid7 {
    type Error = ParseUuid7Error;

    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        if value.get_version_num() == 7 {
            Ok(Uuid7(value))
        } else {
            Err(ParseUuid7Error)
        }
    }
}