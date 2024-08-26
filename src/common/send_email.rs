use std::str::FromStr;

use resend_rs::{types::CreateEmailBaseOptions, Resend};

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

pub struct NetmateEmail(Email);

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

pub struct Subject(String);

pub struct ParseSubjectError;

// RFC 5322 で推奨される一行当たりの最大文字数(1-127までのUS-ASCII範囲が前提)
// https://datatracker.ietf.org/doc/html/rfc5322#section-2.1.1
const MAX_LINE_LENGTH: usize = 78;

impl FromStr for Subject {
    type Err = ParseSubjectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.as_bytes().len() < MAX_LINE_LENGTH {
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