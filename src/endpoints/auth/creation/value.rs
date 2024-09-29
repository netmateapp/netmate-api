use crate::{common::{auth::{one_time_token::OneTimeToken, password::PasswordHash}, email::address::Email, profile::{birth_year::BirthYear, language::Language, region::Region}}, helper::redis::{namespace::Namespace, namespace::NAMESPACE_SEPARATOR}};

const TOKEN_ENTROPY_BITS: usize = 120;

pub const PRE_VERIFICATION_ACCOUNTS_NAMESPACE: Namespace = Namespace::of("pav");

pub const PRE_VERFICATION_ACCOUNTS_VALUE_SEPARATOR: char = '$';

pub fn format_key(token: &OneTimeToken) -> String {
    format!("{}{}{}", PRE_VERIFICATION_ACCOUNTS_NAMESPACE, NAMESPACE_SEPARATOR, token)
}

pub fn format_value(email: &Email, password_hash: &PasswordHash, birth_year: BirthYear, region: Region, language: Language) -> String {
    format!(
        "{}{}{}{}{}{}{}{}{}",
        email,
        PRE_VERFICATION_ACCOUNTS_VALUE_SEPARATOR,
        password_hash,
        PRE_VERFICATION_ACCOUNTS_VALUE_SEPARATOR,
        u16::from(birth_year),
        PRE_VERFICATION_ACCOUNTS_VALUE_SEPARATOR,
        u8::from(region),
        PRE_VERFICATION_ACCOUNTS_VALUE_SEPARATOR,
        u8::from(language)
    )
}