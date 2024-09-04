use std::str::FromStr;

use thiserror::Error;

use crate::{common::language::Language, translation::{ja, us_en}};

use super::address::Email;

pub struct SenderName(String);

impl SenderName {
    pub fn by(language: &Language) -> Self {
        let sender_name = match language {
            Language::Japanese => ja::email::SENDER_NAME,
            _ => us_en::email::SENDER_NAME,
        };
        Ok(sender_name)
    }

    pub fn value(&self) -> &String {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct NetmateEmail(Email);

impl NetmateEmail {
    pub fn value(&self) -> &Email {
        &self.0
    }
}

#[derive(Debug, PartialEq, Error)]
#[error("ドメインのメールアドレスの形式を満たしませんでした")]
pub struct ParseNetmateEmailError;

// ローカル部だけ独立して定義する形では`Email`と同じ検証処理を記述することになるため、
// 原則として`Email`を基にした生成のみ許可する
impl TryFrom<Email> for NetmateEmail {
    type Error = ParseNetmateEmailError;

    fn try_from(value: Email) -> Result<Self, Self::Error> {
        if value.value().ends_with(".netmate.app") {
            Ok(NetmateEmail(value.value().clone()))
        } else {
            Err(ParseNetmateEmailError)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Subject(String);

impl Subject {
    pub fn value(&self) -> &String {
        &self.0
    }
}

#[derive(Debug, PartialEq, Error)]
#[error("件名の形式を満たしませんでした")]
pub struct ParseSubjectError;

// RFC 5322 で推奨される一行当たりの最大文字数(1-127までのUS-ASCII範囲が前提)
// https://datatracker.ietf.org/doc/html/rfc5322#section-2.1.1
const MAX_SUBJECT_LENGTH: usize = 78;

impl FromStr for Subject {
    type Err = ParseSubjectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_empty() && s.as_bytes().len() <= MAX_SUBJECT_LENGTH {
            Ok(Subject(String::from(s)))
        } else {
            Err(ParseSubjectError)
        }
    }
}

pub struct HtmlContent(String);

impl HtmlContent {
    pub fn new(s: &str) -> Self {
        Self(String::from(s))
    }

    pub fn value(&self) -> &String {
        &self.0
    }
}

pub struct PlainText(String);

impl PlainText {
    pub fn new(s: &str) -> Self {
        Self(String::from(s))
    }

    pub fn value(&self) -> &String {
        &self.0
    }
}


// 長い行は折り返されるため、Bodyには制限を適用しない
pub struct Body(HtmlContent, PlainText);

impl Body {
    pub fn new(html_content: &str, plain_text: &str) -> Self {
        Self(HtmlContent::new(html_content), PlainText::new(plain_text))
    }

    pub fn html_content(&self) -> &HtmlContent {
        &self.0
    }

    pub fn plain_text(&self) -> &PlainText {
        &self.1
    }
}

pub(crate) trait EmailSender {
    async fn send(from: &NetmateEmail, to: &Email, sender_name: &SenderName, subject: &Subject, body: &Body) -> Result<(), EmailSendFailed>;
}

#[derive(Debug, thiserror::Error)]
#[error("メールの送信に失敗しました")]
pub struct EmailSendFailed(#[source] pub anyhow::Error);

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::common::email::{send::{NetmateEmail, ParseNetmateEmailError, ParseSubjectError, Subject, MAX_SUBJECT_LENGTH}, address::Email};

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
        assert_eq!(Subject::from_str(&"あ".repeat(MAX_SUBJECT_LENGTH + 1)), Err(ParseSubjectError));
    }
}