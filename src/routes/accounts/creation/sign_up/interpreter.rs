use std::{str::FromStr, sync::{Arc, LazyLock}};

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{birth_year::BirthYear, email::{address::Email, resend::ResendEmailSender, send::{Body, EmailSender, HtmlContent, NetmateEmail, PlainText, SenderName, Subject}}, fallible::Fallible, language::Language, password::PasswordHash, region::Region}, cql, helper::{error::InitError, scylla::prep}, routes::accounts::creation::value::OneTimeToken, translation::{ja, us_en}};

use super::dsl::{SignUp, SignUpError};

pub struct SignUpImpl {
    session: Arc<Session>,
    select_id: Arc<PreparedStatement>,
    insert_account_creation_application: Arc<PreparedStatement>,
}

impl SignUpImpl {
    pub async fn try_new(
        session: Arc<Session>,
    ) -> Result<Self, InitError<SignUpImpl>> {
        let select_id = prep::<InitError<SignUpImpl>>(
            &session,
            cql!("SELECT id FROM accounts_by_email WHERE email = ? LIMIT 1")
        ).await?;

        let insert_account_creation_application = prep::<InitError<SignUpImpl>>(
            &session,
            cql!("INSERT INTO account_creation_applications (ottoken, email, password_hash, birth_year, region, language) VALUES (?, ?, ?, ?, ?, ?) USING TTL 86400")
        ).await?;

        Ok(Self { session, select_id, insert_account_creation_application })
    }
}

static AUTHENTICATION_EMAIL_ADDRESS: LazyLock<NetmateEmail> = LazyLock::new(|| NetmateEmail::try_from(Email::from_str("verify-email@account.netmate.app").unwrap()).unwrap());
static JA_AUTHENTICATION_EMAIL_SUBJECT: LazyLock<Subject> = LazyLock::new(|| Subject::from_str(ja::sign_up::AUTHENTICATION_EMAIL_SUBJECT).unwrap());
static US_EN_AUTHENTICATION_EMAIL_SUBJECT: LazyLock<Subject> = LazyLock::new(|| Subject::from_str(us_en::sign_up::AUTHENTICATION_EMAIL_SUBJECT).unwrap());

impl SignUp for SignUpImpl {
    async fn is_available_email(&self, email: &Email) -> Fallible<bool, SignUpError> {
        let res = self.session
            .execute_unpaged(&self.select_id, (email.value(), ))
            .await;

        match res {
            Ok(qr) => match qr.rows() {
                // ここの正当性が自動テストで保証されていない
                Ok(v) => Ok(!v.is_empty()),
                Err(e) => Err(SignUpError::PotentiallyUnavailableEmail(e.into()))
            },
            Err(e) => Err(SignUpError::PotentiallyUnavailableEmail(e.into()))
        }
    }

    async fn apply_to_create_account(&self, email: &Email, pw_hash: &PasswordHash, birth_year: &BirthYear, region: &Region, language: &Language, token: &OneTimeToken) -> Result<(), SignUpError> {
        let birth_year = i16::from(*birth_year);
        let region = i8::from(*region);
        let language = i8::from(*language);

        self.session
            .execute_unpaged(&self.insert_account_creation_application, (token.value(), email.value(), pw_hash.value(), birth_year, region, language))
            .await
            .map(|_| ())
            .map_err(|e| SignUpError::ApplicationFailed(e.into()))
    }

    async fn send_verification_email(&self, email: &Email, language: &Language, token: &OneTimeToken) -> Result<(), SignUpError> {
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

        ResendEmailSender::send(&*AUTHENTICATION_EMAIL_ADDRESS, email, &sender_name, &subject, &body)
            .await
            .map_err(|e| SignUpError::AuthenticationEmailSendFailed(e.into()))
    }
}