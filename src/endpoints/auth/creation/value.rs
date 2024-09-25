use redis::{RedisWrite, ToRedisArgs};

use crate::{common::{birth_year::BirthYear, email::address::Email, language::Language, password::PasswordHash, region::Region, token::{calc_entropy_bytes, Token}}, helper::redis::{Namespace, NAMESPACE_SEPARATOR}};

const TOKEN_ENTROPY_BITS: usize = 120;

pub type OneTimeToken = Token<{calc_entropy_bytes(TOKEN_ENTROPY_BITS)}>;

pub const PRE_VERIFICATION_ACCOUNTS_NAMESPACE: Namespace = Namespace::of("pav");

pub const PRE_VERFICATION_ACCOUNTS_VALUE_SEPARATOR: char = '$';

pub struct PreVerificationAccountKey<'a>(&'a OneTimeToken);

impl<'a> PreVerificationAccountKey<'a> {
    pub fn new(token: &'a OneTimeToken) -> Self {
        Self(token)
    }

    fn format(&self) -> String {
        format!("{}{}{}", PRE_VERIFICATION_ACCOUNTS_NAMESPACE, NAMESPACE_SEPARATOR, self.0)
    }
}

impl<'a> ToRedisArgs for PreVerificationAccountKey<'a> {
    fn write_redis_args<W: ?Sized + RedisWrite>(&self, out: &mut W) {
        self.format().write_redis_args(out);
    }
}

pub struct PreVerificationAccountValue<'a, 'b>(&'a Email, &'b PasswordHash, BirthYear, Region, Language);

impl<'a, 'b> PreVerificationAccountValue<'a, 'b> {
    pub fn new(email: &'a Email, password_hash: &'b PasswordHash, birth_year: BirthYear, region: Region, language: Language) -> Self {
        Self(email, password_hash, birth_year, region, language)
    }

    fn format(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}{}",
            self.0,
            PRE_VERFICATION_ACCOUNTS_VALUE_SEPARATOR,
            self.1,
            PRE_VERFICATION_ACCOUNTS_VALUE_SEPARATOR,
            u16::from(self.2),
            PRE_VERFICATION_ACCOUNTS_VALUE_SEPARATOR,
            u8::from(self.3),
            PRE_VERFICATION_ACCOUNTS_VALUE_SEPARATOR,
            u8::from(self.4)
        )
    }
}

impl<'a, 'b> ToRedisArgs for PreVerificationAccountValue<'a, 'b> {
    fn write_redis_args<W: ?Sized + RedisWrite>(&self, out: &mut W) {
        self.format().write_redis_args(out);
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{common::{birth_year::BirthYear, email::address::Email, language::Language, password::PasswordHash, region::Region, token::Token}, endpoints::auth::creation::value::{PreVerificationAccountKey, PreVerificationAccountValue, PRE_VERFICATION_ACCOUNTS_VALUE_SEPARATOR, PRE_VERIFICATION_ACCOUNTS_NAMESPACE}, helper::redis::NAMESPACE_SEPARATOR};


    #[test]
    fn test_format_pre_verification_account_key() {
        let token = Token::gen();
        let key = PreVerificationAccountKey::new(&token);

        assert_eq!(key.format(), format!("{}{}{}", PRE_VERIFICATION_ACCOUNTS_NAMESPACE, NAMESPACE_SEPARATOR, token));
    }

    #[test]
    fn test_format_pre_verification_account_value() {
        let email = Email::from_str("email@example.com").unwrap();
        let password_hash = PasswordHash::new_unchecked("");
        let birth_year = BirthYear::try_from(2000u16).unwrap();
        let region = Region::Japan;
        let language = Language::Japanese;

        let value = PreVerificationAccountValue::new(&email, &password_hash, birth_year, region, language);
        assert_eq!(
            value.format(),
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
        );
    }
}