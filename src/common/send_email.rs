use std::str::FromStr;

use resend_rs::{types::CreateEmailBaseOptions, Resend};
use thiserror::Error;

use super::{email::Email, language::Language};

pub enum SenderNameLocale {
    Japanese,
    English,
}

impl SenderNameLocale {
    pub fn expressed_in(language: &Language) -> SenderNameLocale {
        match language {
            Language::Japanese => SenderNameLocale::Japanese,
            _ => SenderNameLocale::English
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct NetmateEmail(Email);

#[derive(Debug, PartialEq, Error)]
#[error("ドメインのメールアドレスの形式を満たしませんでした")]
pub struct ParseNetmateEmailError;

impl TryFrom<Email> for NetmateEmail {
    type Error = ParseNetmateEmailError;

    fn try_from(value: Email) -> Result<Self, Self::Error> {
        if value.value().ends_with(".netmate.app") {
            Ok(NetmateEmail(value))
        } else {
            Err(ParseNetmateEmailError)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Subject(String);

#[derive(Debug, PartialEq, Error)]
#[error("件名の形式を満たしませんでした")]
pub struct ParseSubjectError;

// RFC 5322 で推奨される一行当たりの最大文字数(1-127までのUS-ASCII範囲が前提)
// https://datatracker.ietf.org/doc/html/rfc5322#section-2.1.1
const MAX_LINE_LENGTH: usize = 78;

impl FromStr for Subject {
    type Err = ParseSubjectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_empty() && s.as_bytes().len() <= MAX_LINE_LENGTH {
            Ok(Subject(String::from(s)))
        } else {
            Err(ParseSubjectError)
        }
    }
}

// 長い行は折り返されるため、Bodyには制限を適用しない
pub struct Body {
    html_content: String,
    plain_text: String,
}

impl Body {
    pub fn new(html_content: &str, plain_text: &str) -> Self {
        Self { html_content: String::from(html_content), plain_text: String::from(plain_text) }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("メールの送信に失敗しました")]
pub struct EmailSendFailed(#[source] anyhow::Error);

pub trait TransactionalEmailService {
    async fn send(sender_name: &SenderNameLocale, from: &NetmateEmail, to: &Email, subject: &Subject, body: &Body) -> Result<(), EmailSendFailed>;
}

pub struct ResendEmailService;

impl TransactionalEmailService for ResendEmailService {
    async fn send(sender_name: &SenderNameLocale, from: &NetmateEmail, to: &Email, subject: &Subject, body: &Body) -> Result<(), EmailSendFailed>{
        let resend = Resend::new("");

        let sender_name = match sender_name {
            SenderNameLocale::Japanese => "ネットメイト",
            SenderNameLocale::English => "Netmate"
        };

        let from = format!("{} <{}>", sender_name, from.0.value());
        let to = [to.value()];

        let email = CreateEmailBaseOptions::new(from, to, &subject.0)
            .with_html(&body.html_content)
            .with_text(&body.plain_text);

        resend.emails
            .send(email)
            .await
            .map(|_| ())
            .map_err(|e| EmailSendFailed(e.into()))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::common::{email::Email, send_email::{NetmateEmail, ParseNetmateEmailError, ParseSubjectError, Subject, MAX_LINE_LENGTH}};

    #[test]
    fn netmate_email() {
        assert!(NetmateEmail::try_from(Email::from_str("verify-email@account.netmate.app").unwrap()).is_ok());
    }

    #[test]

    fn non_netmate_email() {
        assert_eq!(NetmateEmail::try_from(Email::from_str("verify-email@account.netmate.com").unwrap()), Err(ParseNetmateEmailError));
    }

    #[test]
    fn normal_subject() {
        assert!(Subject::from_str("メールアドレスを確認してください").is_ok());
    }

    #[test]
    fn empty_subject() {
        assert_eq!(Subject::from_str(""), Err(ParseSubjectError));
    }

    #[test]
    fn subject_too_long() {
        assert_eq!(Subject::from_str(&"あ".repeat(MAX_LINE_LENGTH + 1)), Err(ParseSubjectError));
    }
}