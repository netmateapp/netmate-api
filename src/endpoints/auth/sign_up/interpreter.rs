use std::{str::FromStr, sync::{Arc, LazyLock}};

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{birth_year::BirthYear, email::{address::Email, resend::ResendEmailSender, send::{Body, EmailSender, HtmlContent, NetmateEmail, PlainText, SenderName, Subject}}, fallible::Fallible, id::account_id::AccountId, language::Language, one_time_token::OneTimeToken, password::PasswordHash, region::Region}, helper::{error::InitError, scylla::prepare}, translation::{ja, us_en}};

use super::dsl::{SignUp, SignUpError};

pub struct SignUpImpl {
    db: Arc<Session>,
    select_account_id: Arc<PreparedStatement>,
    insert_pre_verification_account: Arc<PreparedStatement>,
}

impl SignUpImpl {
    pub async fn try_new(db: Arc<Session>) -> Result<Self, InitError<SignUpImpl>> {
        fn handle_error<E: Into<anyhow::Error>>(e: E) -> InitError<SignUpImpl> {
            InitError::new(e.into())
        }

        let select_account_id = prepare(&db, "SELECT id FROM accounts WHERE email = ? LIMIT 1").await?;

        let insert_pre_verification_account = prepare(&db, "INSERT INTO pre_verification_accounts (one_time_token, email, password_hash, birth_year, region, language) VALUES (?, ?, ?, ?, ?, ?) USING TTL ?").await?;

        Ok(Self { db, select_account_id, insert_pre_verification_account })
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

    async fn apply_to_create_account(&self, email: &Email, pw_hash: &PasswordHash, birth_year: BirthYear, region: Region, language: Language, token: &OneTimeToken) -> Result<(), SignUpError> {
        self.db
            .execute_unpaged(&self.insert_pre_verification_account, (token, email, pw_hash, birth_year, region, language))
            .await
            .map(|_| ())
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