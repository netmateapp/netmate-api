use std::{error::Error, str::FromStr, sync::Arc};

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{birth_year::{BirthYear, ParseBirthYearError}, email::Email, fallible::Fallible, id::AccountId, language::{Language, ParseLanguageError}, password::PasswordHash, region::{ParseRegionError, Region}}, helper::{error::InitError, scylla::prepare}, routes::accounts::creation::sign_up::value::OneTimeToken};

use super::dsl::{VerifyEmail, VerifyEmailError};

pub struct VerifyEmailImpl {
    session: Arc<Session>,
    select_account_creation_application: Arc<PreparedStatement>,
    insert_account: Arc<PreparedStatement>,
    delete_account_creation_application: Arc<PreparedStatement>,
}

impl VerifyEmailImpl {
    pub async fn try_new(session: Arc<Session>) -> Result<Self, InitError<VerifyEmailImpl>> {
        let select_account_creation_application = prepare::<InitError<VerifyEmailImpl>>(
            &session,
            "SELECT email, password_hash, birth_year, region, language FROM account_creation_applications WHERE ottoken = ?"
        ).await?;

        let insert_account = prepare::<InitError<VerifyEmailImpl>>(
            &session,
            "INSERT INTO accounts (id, email, password_hash, birth_year, region, language) VALUES (?, ?, ?, ?, ?, ?) IF NOT EXISTS"
        ).await?;

        let delete_account_creation_application = prepare::<InitError<VerifyEmailImpl>>(
            &session,
            "DELETE FROM account_creation_applications WHERE code = ?"
        ).await?;

        Ok(Self { session, select_account_creation_application, insert_account, delete_account_creation_application })
    }
}

impl VerifyEmail for VerifyEmailImpl {
    async fn retrieve_account_creation_application_by(&self, token: &OneTimeToken) -> Fallible<(Email, PasswordHash, BirthYear, Region, Language), VerifyEmailError> {
        let res = self.session
            .execute(&self.select_account_creation_application, (token.value(), ))
            .await
            .map_retrieve_err()?;

        let (email, password_hash, birth_year, region, language) = res.first_row_typed::<(String, String, i16, i8, i8)>()
            .map_retrieve_err()?;

        let email = Email::from_str(email.as_str())
            .map_retrieve_err()?;
        let password_hash = PasswordHash::from_str(password_hash.as_str())
            .map_retrieve_err()?;
        let birth_year = i16_to_birth_year(birth_year)
            .map_retrieve_err()?;
        let region = i8_to_region(region)
            .map_retrieve_err()?;
        let language = i8_to_language(language)
            .map_retrieve_err()?;

        Ok((email, password_hash, birth_year, region, language))
    }

    async fn create_account(&self, account_id: &AccountId, email: &Email, password_hash: &PasswordHash, birth_year: &BirthYear, region: &Region, language: &Language) -> Fallible<(), VerifyEmailError> {
        let birth_year = birth_year_to_i16(birth_year);
        let region = region_to_i8(region);
        let language = language_to_i8(language);
        
        self.session
            .execute(&self.insert_account, (account_id.value(), email.value(), password_hash.value(), birth_year, region, language))
            .await
            .map(|_| ())
            .map_err(|e| VerifyEmailError::CreateAccountFailed(e.into()))
    }

    async fn delete_account_creation_application_by(&self, token: &OneTimeToken) -> Fallible<(), VerifyEmailError> {
        self.session
            .execute(&self.delete_account_creation_application, (token.value(), ))
            .await
            .map(|_| ())
            .map_err(|e| VerifyEmailError::DeleteAccountCreationApplicationFailed(e.into()))
    }
}

fn i16_to_birth_year(n: i16) -> Result<BirthYear, ParseBirthYearError> {
    BirthYear::try_from(n as u16)
}

fn i8_to_region(n: i8) -> Result<Region, ParseRegionError> {
    Region::try_from(n as u8)
}

fn i8_to_language(n: i8) -> Result<Language, ParseLanguageError> {
    Language::try_from(n as u8)
}

// sign_up/interpreter.rsと重複している
fn birth_year_to_i16(birth_year: &BirthYear) -> i16 {
    u16::from(*birth_year) as i16
}

fn region_to_i8(region: &Region) -> i8 {
    u8::from(*region) as i8
}

fn language_to_i8(language: &Language) -> i8 {
    u8::from(*language) as i8
}

trait RetrieveErrorMapper<T> {
    fn map_retrieve_err(self) -> Result<T, VerifyEmailError>;
}

impl<T, E: Error + Send + Sync + 'static> RetrieveErrorMapper<T> for Result<T, E> {
    fn map_retrieve_err(self) -> Result<T, VerifyEmailError> {
        self.map_err(|e| VerifyEmailError::RetrieveAccountCreationApplicationFailed(e.into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{common::{birth_year::{BirthYear, MAX_BIRTH_YEAR, MIN_BIRTH_YEAR}, language::Language, region::Region}, routes::accounts::creation::verify_email::interpreter::{birth_year_to_i16, i16_to_birth_year, i8_to_language, i8_to_region, language_to_i8, region_to_i8}};

    // `retrieve_account_creation_application`関連のテスト
    #[test]
    fn to_birth_year() {
        assert_eq!(i16_to_birth_year(0).unwrap(), BirthYear::try_from(0).unwrap());
        assert_eq!(i16_to_birth_year(MIN_BIRTH_YEAR as i16).unwrap(), BirthYear::try_from(MIN_BIRTH_YEAR).unwrap());
        assert_eq!(i16_to_birth_year(*MAX_BIRTH_YEAR as i16).unwrap(), BirthYear::try_from(*MAX_BIRTH_YEAR).unwrap());
    }

    #[test]
    fn to_region() {
        assert_eq!(i8_to_region(0).unwrap(), Region::Afghanistan);
        assert_eq!(i8_to_region(197u8 as i8).unwrap(), Region::Zimbabwe);
    }

    #[test]
    fn to_language() {
        assert_eq!(i8_to_language(0).unwrap(), Language::AmericanEnglish);
        assert_eq!(i8_to_language(3 as i8).unwrap(), Language::TaiwaneseMandarin);
    }

    // `create_account`関連のテスト
    #[test]
    fn from_birth_year() {
        let unspecified = BirthYear::try_from(0).unwrap();
        assert_eq!(birth_year_to_i16(&unspecified) as u16, 0);

        let min_birth_year = BirthYear::try_from(MIN_BIRTH_YEAR).unwrap();
        assert_eq!(birth_year_to_i16(&min_birth_year) as u16, MIN_BIRTH_YEAR);

        let max_birth_year = BirthYear::try_from(*MAX_BIRTH_YEAR).unwrap();
        assert_eq!(birth_year_to_i16(&max_birth_year) as u16, *MAX_BIRTH_YEAR);
    }

    #[test]
    fn from_region() {
        assert_eq!(region_to_i8(&Region::Afghanistan) as u8, 0);
        assert_eq!(region_to_i8(&Region::Zimbabwe) as u8, 197);
    }

    #[test]
    fn from_language() {
        assert_eq!(language_to_i8(&Language::AmericanEnglish) as u8, 0);
        assert_eq!(language_to_i8(&Language::TaiwaneseMandarin) as u8, 3);
    }
}