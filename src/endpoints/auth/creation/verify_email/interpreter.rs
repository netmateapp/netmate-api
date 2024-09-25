use std::{str::{FromStr, SplitN}, sync::Arc};

use redis::cmd;
use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{birth_year::BirthYear, email::address::Email, fallible::Fallible, id::account_id::AccountId, language::Language, one_time_token::OneTimeToken, password::PasswordHash, region::Region}, endpoints::auth::creation::value::{PreVerificationAccountKey, PRE_VERFICATION_ACCOUNTS_VALUE_SEPARATOR}, helper::{error::InitError, redis::{conn, Pool}, scylla::prepare}};

use super::dsl::{VerifyEmail, VerifyEmailError};

pub struct VerifyEmailImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    insert_account: Arc<PreparedStatement>,
}

impl VerifyEmailImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<VerifyEmailImpl>> {
        let insert_account = prepare(&db, "INSERT INTO accounts (id, email, password_hash, birth_year, region, language) VALUES (?, ?, ?, ?, ?, ?) IF NOT EXISTS").await?;

        Ok(Self { db, cache, insert_account })
    }
}

impl VerifyEmail for VerifyEmailImpl {
    async fn retrieve_account_creation_application_by(&self, token: &OneTimeToken) -> Fallible<Option<(Email, PasswordHash, BirthYear, Region, Language)>, VerifyEmailError> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> VerifyEmailError {
            VerifyEmailError::RetrieveAccountCreationApplicationFailed(e.into())
        }

        fn parse<T, E: Into<anyhow::Error>>(parts: &mut SplitN<'_, char>, parser: impl FnOnce(&str) -> Result<T, E>, value_name: &str) -> Result<T, VerifyEmailError> {
            parts.next()
                .ok_or_else(|| handle_error(anyhow::anyhow!("{}データがありません", value_name)))
                .and_then(|s| parser(s).map_err(handle_error))
        }

        fn parse_num<N, T, E1: Into<anyhow::Error>, E2: Into<anyhow::Error>>(parts: &mut SplitN<'_, char>, num_parser: impl FnOnce(&str) -> Result<N, E1>, parser: impl FnOnce(N) -> Result<T, E2>, value_name: &str) -> Result<T, VerifyEmailError> {
            parse(parts, |s| num_parser(s), value_name)
                .map(parser)?
                .map_err(handle_error)
        }

        let mut conn = conn(&self.cache, handle_error).await?;
        
        cmd("GET")
            .arg(PreVerificationAccountKey::new(token))
            .query_async::<Option<String>>(&mut *conn)
            .await
            .map_err(handle_error)
            .transpose()
            .map(|o| o.and_then(|s| {
                let mut parts = s.splitn(5, PRE_VERFICATION_ACCOUNTS_VALUE_SEPARATOR);

                let email = parse(&mut parts, Email::from_str, "メールアドレス")?;
                let password_hash = parse(&mut parts, PasswordHash::from_str, "パスワードハッシュ")?;
                let birth_year = parse_num(&mut parts, str::parse::<u16>, BirthYear::try_from, "生年")?;
                let region = parse_num(&mut parts, str::parse::<u8>, Region::try_from, "地域")?;
                let language = parse_num(&mut parts, str::parse::<u8>, Language::try_from, "言語")?;

                Ok((email, password_hash, birth_year, region, language))
            }))
            .transpose()
    }

    async fn create_account(&self, account_id: AccountId, email: &Email, password_hash: &PasswordHash, birth_year: BirthYear, region: Region, language: Language) -> Fallible<(), VerifyEmailError> {
        self.db
            .execute_unpaged(&self.insert_account, (account_id, email, password_hash, birth_year, region, language))
            .await
            .map(|_| ())
            .map_err(|e| VerifyEmailError::CreateAccountFailed(e.into()))
    }

    async fn delete_account_creation_application_by(&self, token: &OneTimeToken) -> Fallible<(), VerifyEmailError> {
        let mut conn = conn(&self.cache, |e| VerifyEmailError::DeleteAccountCreationApplicationFailed(e.into())).await?;

        cmd("DEL")
            .arg(PreVerificationAccountKey::new(token))
            .exec_async(&mut *conn)
            .await
            .map_err(|e| VerifyEmailError::DeleteAccountCreationApplicationFailed(e.into()))
    }
}