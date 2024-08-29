use std::sync::Arc;

use scylla::{prepared_statement::PreparedStatement, Session};

use crate::{common::{birth_year::BirthYear, email::Email, fallible::Fallible, language::Language, password::PasswordHash, region::Region, send_email::{Body, NetmateEmail, ResendEmailService, SenderNameLocale, Subject, TransactionalEmailService}}, helper::{error::InitError, scylla::prepare}, translation::{ja, us_en}};

use super::{dsl::{SignUp, SignUpError}, value::OneTimeToken};

pub struct SignUpImpl {
    session: Arc<Session>,
    exists_by_email: Arc<PreparedStatement>,
    insert_account_creation_application: Arc<PreparedStatement>,
}

impl SignUpImpl {
    pub async fn try_new(
        session: Arc<Session>,
    ) -> Result<Self, InitError<SignUpImpl>> {
        let exists_by_email = prepare::<InitError<SignUpImpl>>(
            &session,
            "SELECT id FROM accounts_by_email WHERE email = ?"
        ).await?;

        let insert_account_creation_application = prepare::<InitError<SignUpImpl>>(
            &session,
            "INSERT INTO account_creation_applications (token, email, password_hash, birth_year, region, language) VALUES (?, ?, ?, ?, ?, ?) USING TTL 86400"
        ).await?;

        Ok(Self { session, exists_by_email, insert_account_creation_application })
    }
}

const AUTHENTICATION_EMAIL_ADDRESS: &str = "verify-email@account.netmate.app";

impl SignUp for SignUpImpl {
    async fn is_available_email(&self, email: &Email) -> Fallible<bool, SignUpError> {
        let res = self.session
            .execute(&self.exists_by_email, (email.value(), ))
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
        // ここを検証しないとテストの意味がない
        let encoded_birth_year: i16 = (*birth_year).into();
        let encoded_region: i8 = (*region).into();
        let encoded_language: i8 = (*language).into();

        self.session
            .execute(&self.insert_account_creation_application, (token.value(), email.value(), pw_hash.value(), encoded_birth_year, encoded_region, encoded_language,))
            .await
            .map(|_| ())
            .map_err(|e| SignUpError::ApplicationFailed(e.into()))
    }

    async fn send_verification_email(&self, email: &Email, language: &Language, token: &OneTimeToken) -> Result<(), SignUpError> {
        let sender_name = &SenderNameLocale::expressed_in(language);

        // ユーザーの設定言語に応じたテキストを取得する
        let (subject, html_content, plain_text) = match language {
            Language::Japanese => (ja::sign_up::AUTHENTICATION_EMAIL_SUBJECT, ja::sign_up::ATUHENTICATION_EMAIL_BODY_HTML, ja::sign_up::AUTHENTICATION_EMAIL_BODY_PLAIN),
            _ => (us_en::sign_up::AUTHENTICATION_EMAIL_SUBJECT, us_en::sign_up::ATUHENTICATION_EMAIL_BODY_HTML, us_en::sign_up::AUTHENTICATION_EMAIL_BODY_PLAIN),
        };

        // `new_unchecked`により生成された値オブジェクトの正当性は自動テストが保証する
        ResendEmailService::send(
            sender_name,
            &NetmateEmail::new_unchecked(AUTHENTICATION_EMAIL_ADDRESS),
            email,
            &Subject::new_unchecked(subject),
            &Body::new(
                &html_content.replace("{token}", token.value()),
                &plain_text.replace("{token}", token.value())
            )
        )
            .await
            .map_err(|e| SignUpError::AuthenticationEmailSendFailed(e.into()))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{common::{email::Email, send_email::{NetmateEmail, Subject}}, translation::{ja, us_en}};

    use super::{SignUpError, AUTHENTICATION_EMAIL_ADDRESS};

    #[test]
    fn sender_email() {
        let from = match Email::from_str(AUTHENTICATION_EMAIL_ADDRESS) {
            Ok(email) => match NetmateEmail::try_from(email) {
                Ok(ne) => Ok(ne),
                Err(e) => Err(SignUpError::AuthenticationEmailSendFailed(e.into())),
            },
            Err(e) => Err(SignUpError::AuthenticationEmailSendFailed(e.into()))
        };
        assert!(from.is_ok());
    }

    #[test]
    fn all_language_subjects() {
        let _ = Subject::from_str(ja::sign_up::AUTHENTICATION_EMAIL_SUBJECT);
        let _ = Subject::from_str(us_en::sign_up::AUTHENTICATION_EMAIL_SUBJECT);
    }
}