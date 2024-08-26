use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use scylla::{prepared_statement::PreparedStatement, Session};
use serde::Deserialize;
use tokio::task;

use crate::{common::{birth_year::BirthYear, email::Email, language::Language, password::{Password, PasswordHash}, region::Region, send_email::{Body, NetmateEmail, ResendEmailService, SenderNameLocale, Subject, TransactionalEmailService}}, translation::{ja, us_en}};

use super::{dsl::{Fallible, SignUp, SignUpError}, value::OneTimeToken};

pub async fn sign_up_route(db: Arc<Session>) -> Router {
    let exists_by_email = db.prepare("SELECT id FROM accounts_by_email WHERE email = ?").await.unwrap();
    let insert_creation_application = db.prepare("INSERT INTO account_creation_applications (email, password_hash, region, language, birth_year, code) VALUES (?, ?, ?, ?, ?, ?) USING TTL 86400").await.unwrap();

    let routine = SignUpImpl {
        session: db,
        exists_by_email: Arc::new(exists_by_email),
        insert_creation_application: Arc::new(insert_creation_application)
    };

    Router::new()
        .route("/sign_up", post(handler))
        .with_state(Arc::new(routine))
}

// quick exit 対策はここで行い、アプリケーションには波及させない
// 返された成否情報をもとにロギング
pub async fn handler(
    State(routine): State<Arc<SignUpImpl>>,
    Json(payload): Json<Payload>,
) -> impl IntoResponse {
    task::spawn(async move {
        // 結果をログに記録
        routine.sign_up(&payload.email, &payload.password, &payload.birth_year, &payload.region, &payload.language).await;
    });
    StatusCode::OK
}

#[derive(Deserialize)]
pub struct Payload {
    pub email: Email,
    pub password: Password,
    pub region: Region,
    pub language: Language,
    pub birth_year: BirthYear,
}

pub struct SignUpImpl {
    session: Arc<Session>,
    exists_by_email: Arc<PreparedStatement>,
    insert_creation_application: Arc<PreparedStatement>,
}

const AUTHENTICATION_EMAIL_ADDRESS: &str = "verify-email@account.netmate.app";

impl SignUp for SignUpImpl {
    async fn is_available_email(&self, email: &Email) -> Fallible<bool, SignUpError> {
        let res = self.session
            // ここに clone() が必要で、clone()を強制する設計が必要
            .execute(&self.exists_by_email, (email.value(), ))
            .await;

        match res {
            Ok(qr) => match qr.rows() {
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

        let res = self.session
            .execute(&self.insert_creation_application, (token.value(), email.value(), pw_hash.value(), encoded_birth_year, encoded_region, encoded_language,))
            .await;

        res.map(|_| ())
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