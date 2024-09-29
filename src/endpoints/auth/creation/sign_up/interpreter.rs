use std::{str::FromStr, sync::{Arc, LazyLock}};

use redis::cmd;
use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{email::{address::Email, resend::ResendEmailSender, send::{Body, EmailSender, HtmlContent, NetmateEmail, PlainText, SenderName, Subject}}, fallible::Fallible, one_time_token::OneTimeToken, password::PasswordHash, profile::{account_id::AccountId, birth_year::BirthYear, language::Language, region::Region}}, endpoints::auth::creation::value::{format_key, format_value}, helper::{error::InitError, redis::{conn, Pool}, scylla::prepare}, translation::{ja, us_en}};

use super::dsl::{ApplicationExpirationSeconds, SignUp, SignUpError};

pub struct SignUpImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
    select_account_id: Arc<PreparedStatement>,
}

impl SignUpImpl {
    pub async fn try_new(db: Arc<Session>, cache: Arc<Pool>) -> Result<Self, InitError<SignUpImpl>> {
        let select_account_id = prepare(&db, "SELECT id FROM accounts WHERE email = ? LIMIT 1 BYPASS CACHE").await?;
        Ok(Self { db, cache, select_account_id })
    }
}

static AUTHENTICATION_EMAIL_ADDRESS: LazyLock<NetmateEmail> = LazyLock::new(|| NetmateEmail::try_from(Email::from_str("verify-email@account.netmate.app").unwrap()).unwrap());
static JA_AUTHENTICATION_EMAIL_SUBJECT: LazyLock<Subject> = LazyLock::new(|| Subject::from_str(ja::sign_up::AUTHENTICATION_EMAIL_SUBJECT).unwrap());
static US_EN_AUTHENTICATION_EMAIL_SUBJECT: LazyLock<Subject> = LazyLock::new(|| Subject::from_str(us_en::sign_up::AUTHENTICATION_EMAIL_SUBJECT).unwrap());

impl SignUp for SignUpImpl {
    async fn is_available_email(&self, email: &Email) -> Fallible<bool, SignUpError> {
        self.db
            .execute_unpaged(&self.select_account_id, (email, ))
            .await
            .map_err(|e| SignUpError::PotentiallyUnavailableEmail(e.into()))?
            .maybe_first_row_typed::<(AccountId, )>()
            .map(|res| res.is_some())
            .map_err(|e| SignUpError::PotentiallyUnavailableEmail(e.into()))
    }

    async fn apply_to_create_account(&self, email: &Email, password_hash: &PasswordHash, birth_year: BirthYear, region: Region, language: Language, token: &OneTimeToken, expiration: ApplicationExpirationSeconds) -> Result<(), SignUpError> {
        let mut conn = conn(&self.cache, |e| SignUpError::ApplicationFailed(e.into())).await?;

        cmd("SET")
            .arg(format_key(token))
            .arg(format_value(email, password_hash, birth_year, region, language))
            .arg("EX")
            .arg(expiration)
            .exec_async(&mut *conn)
            .await
            .map_err(|e| SignUpError::ApplicationFailed(e.into()))
    }

    async fn send_verification_email(&self, email: &Email, language: Language, token: &OneTimeToken) -> Result<(), SignUpError> {
        let sender_name = SenderName::by(language);

        // ユーザーの設定言語に応じたテキストを取得する
        let (subject, html_content, plain_text) = match language {
            Language::Japanese => (&*JA_AUTHENTICATION_EMAIL_SUBJECT, ja::sign_up::ATUHENTICATION_EMAIL_BODY_HTML, ja::sign_up::AUTHENTICATION_EMAIL_BODY_PLAIN),
            _ => (&*US_EN_AUTHENTICATION_EMAIL_SUBJECT, us_en::sign_up::ATUHENTICATION_EMAIL_BODY_HTML, us_en::sign_up::AUTHENTICATION_EMAIL_BODY_PLAIN),
        };

        let body = Body::new(
            HtmlContent::new(&html_content.replace("{token}", token.value())),
            PlainText::new(&plain_text.replace("{token}", token.value()))
        );

        ResendEmailSender::send(&AUTHENTICATION_EMAIL_ADDRESS, email, &sender_name, subject, &body)
            .await
            .map_err(|e| SignUpError::AuthenticationEmailSendFailed(e.into()))
    }
}