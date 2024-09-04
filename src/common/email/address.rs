use std::{str::FromStr, sync::LazyLock};

use idna::domain_to_ascii;
use regex::Regex;
use serde::{de::{self}, Deserialize};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Email(String);

impl Email {
    pub fn value(&self) -> &String {
        return &self.0;
    }
}

#[derive(Debug, Error)]
#[error("メールアドレスの形式を満たしませんでした")]
pub struct ParseEmailError;

impl FromStr for Email {
    type Err = ParseEmailError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if validate_email(s) {
            Ok(Email(String::from(s)))
        } else {
            Err(ParseEmailError)
        }
    }
}

// メールアドレスの検証は、validatorの処理を流用する
// ただし、ドメイン部でのIPアドレスの使用は、セキュリティの観点から禁止する。
// https://github.com/Keats/validator/blob/99b2191af3baa15fae0274aa65bf94bba621c40a/validator/src/validation/email.rs#L43C1-L79C6
static EMAIL_USER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+\z").unwrap());
static EMAIL_DOMAIN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap());

fn validate_email(s: &str) -> bool {
    if s.is_empty() || !s.contains('@') {
        return false;
    }

    let parts: Vec<&str> = s.rsplitn(2, '@').collect();
    let user_part = parts[1];
    let domain_part = parts[0];

    // 正規表現による検証の前に、メールアドレスの各部分の長さを検証する
    // RFC5321によると、ローカル部分の最大長は64文字
    // ドメイン部分の最大長は255文字
    // https://datatracker.ietf.org/doc/html/rfc5321#section-4.5.3.1.1
    if user_part.chars().count() > 64 || domain_part.chars().count() > 255 {
        return false;
    }

    if !EMAIL_USER_RE.is_match(user_part) {
        return false;
    }

    if !validate_domain_part(domain_part) {
        // まだ[国際化ドメイン](https://en.wikipedia.org/wiki/Internationalized_domain_name)の可能性がある
        return match domain_to_ascii(domain_part) {
            Ok(d) => validate_domain_part(&d),
            Err(_) => false,
        };
    }

    true
}

fn validate_domain_part(domain_part: &str) -> bool {
    EMAIL_DOMAIN_RE.is_match(domain_part)
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Email::from_str(s).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::common::email::address::{validate_email, Email};

    #[test]
    fn normal() {
        assert!(validate_email("email@example.com"));
    }

    #[test]
    fn idn() {
        assert!(validate_email("email@日本語.jp"));
    }

    #[test]
    fn ipv4() {
        assert!(!validate_email("email@[192.0.2.0]"));
    }

    #[test]
    fn ipv6() {
        assert!(!validate_email("email@[3fff:fff:ffff:ffff:ffff:ffff:ffff:ffff]"));
    }

    #[test]
    fn non_ascii_local_part() {
        assert!(!validate_email("メール@example.com"));
    }

    #[test]
    fn deserialize_valid_json() {
        let json = r#""email@example.com""#;
        let email: Email = serde_json::from_str(json).unwrap();
        assert_eq!(email, Email::from_str("email@example.com").unwrap());
    }

    #[test]
    fn deserialize_invalid_json() {
        let json = r#""メール@example.com""#;
        let email = serde_json::from_str::<Email>(json);
        assert!(email.is_err());
    }
}