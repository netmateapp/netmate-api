use std::{collections::HashSet, fs::File, io::{BufRead, BufReader}, str::FromStr, sync::LazyLock};

use argon2::{password_hash::{self, PasswordHasher, SaltString}, Algorithm, Argon2, ParamsBuilder, PasswordVerifier, Version};
use base64::{engine::general_purpose, Engine};
use rand::rngs::OsRng;
use regex::Regex;
use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{de::{self}, Deserialize};
use thiserror::Error;

static ARGON2_CONTEXT: LazyLock<Argon2> = LazyLock::new(|| {
    const MEMORY: u32 = 19 * 1024;
    const ITERATIONS: u32 = 2;
    const DEGREE_OF_PARALLELISM: u32 = 1;

    let params = ParamsBuilder::new()
        .m_cost(MEMORY)
        .t_cost(ITERATIONS)
        .p_cost(DEGREE_OF_PARALLELISM)
        .build()
        .unwrap();

    Argon2::new_with_secret(&*PEPPER, Algorithm::Argon2id, Version::V0x13, params).unwrap()
});

#[derive(Debug, PartialEq)]
pub struct Password(String);

impl Password {
    pub fn value(&self) -> &String {
        &self.0
    }

    pub fn hashed(&self) -> PasswordHash {
        let salt = SaltString::generate(&mut OsRng);

        let phc_format_hash = ARGON2_CONTEXT.hash_password(&self.value().as_bytes(), &salt).unwrap().to_string();
    
        PasswordHash(phc_format_hash)
    }
}

const MIN_PASSWORD_LENGTH: usize = 10;
const MAX_PASSWORD_LENGTH: usize = 1024;

impl FromStr for Password {
    type Err = ParsePasswordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().count() {
            ..MIN_PASSWORD_LENGTH => return Err(ParsePasswordError::TooShort),
            MIN_PASSWORD_LENGTH..=MAX_PASSWORD_LENGTH => (),
            _ => return Err(ParsePasswordError::TooLong)
        }

        if is_unsafe_password(s) {
            return Err(ParsePasswordError::Unsafe);
        }

        Ok(Password(String::from(s)))
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum ParsePasswordError {
    #[error("パスワードが短すぎます")]
    TooShort,
    #[error("パスワードが長すぎます")]
    TooLong,
    #[error("パスワードが安全ではありません")]
    Unsafe,
}

static PHC_FORMAT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\$[a-z0-9-]{1,32}(?:\$v=1[69])?(?:\$m=[1-9][0-9]{0,9},t=[1-9][0-9]{0,9},p=[1-9][0-9]{0,2}(?:,keyid=[a-zA-Z0-9\/+.-]{0,11})?(?:,data=[a-zA-Z0-9\/+.-]{0,43})?)?(?:\$[a-zA-Z0-9\/+.-]{11,64})?(?:\$[a-zA-Z0-9\/+.-]{16,86})?$").unwrap());

// Password("".to_string()).hashed().0 と等価
pub static EMPTY_PASSWORD_HASH: LazyLock<PasswordHash> = LazyLock::new(|| PasswordHash("$argon2id$v=19$m=19456,t=2,p=1$MicFwG/r8ASLXzDKEpvvrw$8f2+X/qZE8RpJT+ietxpWAEd/dIPRLKFDOjB4hlGrpA".to_owned()));

#[derive(Debug, Clone)]
pub struct PasswordHash(String);

impl PasswordHash {
    #[cfg(debug_assertions)]
    pub fn new_unchecked(s: &str) -> Self {
        Self(String::from(s))
    }

    pub fn value(&self) -> &String {
        &self.0
    }

    pub fn verify(&self, password: &Password) -> bool {
        // PHCフォーマットを満たしたもののみがインスタンス化されるため`unwrap`は安全
        let parsed_hash = password_hash::PasswordHash::new(&self.0).unwrap();

        ARGON2_CONTEXT.verify_password(password.value().as_bytes(), &parsed_hash).is_ok()
    }
}

#[derive(Debug, Error)]
#[error("PHCフォーマットを満たしていません")]
pub struct ParsePasswordHashError;

impl FromStr for PasswordHash {
    type Err = ParsePasswordHashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if PHC_FORMAT.is_match(s) {
            Ok(PasswordHash(String::from(s)))
        } else {
            Err(ParsePasswordHashError)
        }
    }
}

impl SerializeValue for PasswordHash {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        self.0.serialize(typ, writer)
    }
}

impl FromCqlVal<Option<CqlValue>> for PasswordHash {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        String::from_cql(cql_val).map(PasswordHash)
    }
}

// 以下はリファクタリングが必要
static PEPPER: LazyLock<[u8; 32]> = LazyLock::new(load_pepper);

fn load_pepper() -> [u8; 32] {
    let base64 = dotenvy::var("PEPPER").unwrap();
    let decoded = general_purpose::STANDARD.decode(base64).unwrap();
    println!("ペッパーを読み込みました。");
    decoded.as_slice().try_into().unwrap()
}

const UNSAFE_PASSWORDS_FILE_PATH: &str = "xato-net-10-million-passwords-filtered-min-10-chars.txt";
static UNSAFE_PASSWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| load_unsafe_passwords(UNSAFE_PASSWORDS_FILE_PATH).unwrap());

fn load_unsafe_passwords(file_path: &str) -> std::io::Result<HashSet<String>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut unsafe_passwords = HashSet::<String>::new();

    for line in reader.lines() {
        let password = line?;
        unsafe_passwords.insert(password);
    }

    println!("{}件の安全でないパスワードを読み込みました。", unsafe_passwords.len());

    Ok(unsafe_passwords)
}

fn is_unsafe_password(password: &str) -> bool {
    UNSAFE_PASSWORDS.contains(password)
}

impl<'de> Deserialize<'de> for Password {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Password::from_str(s).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::common::password::{ParsePasswordError, Password, PasswordHash, MAX_PASSWORD_LENGTH, MIN_PASSWORD_LENGTH};

    #[test]
    fn password_too_short() {
        assert_eq!(Password::from_str(&"a".repeat(MIN_PASSWORD_LENGTH - 1)), Err(ParsePasswordError::TooShort));
    }

    #[test]
    fn password_too_long() {
        assert_eq!(Password::from_str(&"a".repeat(MAX_PASSWORD_LENGTH + 1)), Err(ParsePasswordError::TooLong));
    }

    #[test]
    fn unsafe_password() {
        assert_eq!(Password::from_str("0000000000"), Err(ParsePasswordError::Unsafe));
    }

    #[test]
    fn safe_password() {
        let password: &str = "SCBGpks6FfnCb6R";
        assert_eq!(Password::from_str(password), Ok(Password(String::from(password))));
    }

    #[test]
    fn deserialize_valid_json() {
        let json = r#""SCBGpks6FfnCb6R""#;
        let password: Password = serde_json::from_str(json).unwrap();
        assert_eq!(password, Password(String::from("SCBGpks6FfnCb6R")));
    }

    #[test]
    fn deserialize_invalid_json() {
        let json = r#""0000000000""#;
        let password = serde_json::from_str::<Password>(json);
        assert!(password.is_err());
    }

    #[test]
    fn valid_password_hash() {
        let hash = "$argon2id$v=19$m=65536,t=2,p=1$gZiV/M1gPc22ElAH/Jh1Hw$CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno";
        assert!(PasswordHash::from_str(hash).is_ok());
    }

    #[test]
    fn invalid_password_hash() {
        let hash = "SCBGpks6FfnCb6R";
        assert!(PasswordHash::from_str(hash).is_err());
    }
}