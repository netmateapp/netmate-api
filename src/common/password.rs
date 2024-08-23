use std::{collections::HashSet, fs::File, io::{BufRead, BufReader}, str::FromStr, sync::LazyLock};

use argon2::{password_hash::{PasswordHasher, SaltString}, Algorithm, Argon2, ParamsBuilder, Version};
use base64::{engine::general_purpose, Engine};
use rand::rngs::OsRng;

#[derive(Debug, PartialEq)]
pub struct Password(String);

impl Password {
    pub fn value(&self) -> &String {
        &self.0
    }

    pub fn hashed(&self) -> PasswordHash {
        hash_password(&self)
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

#[derive(Debug, PartialEq)]
pub enum ParsePasswordError {
    TooShort,
    TooLong,
    Unsafe,
}

pub struct PasswordHash(String);

impl PasswordHash {
    pub fn value(&self) -> &String {
        &self.0
    }
}

static PEPPER: LazyLock<[u8; 32]> = LazyLock::new(load_pepper);

fn load_pepper() -> [u8; 32] {
    let base64 = dotenvy::var("PEPPER").unwrap();
    let decoded = general_purpose::STANDARD.decode(base64).unwrap();
    decoded.as_slice().try_into().unwrap()
}

pub fn hash_password(password: &Password) -> PasswordHash {
    let salt = SaltString::generate(&mut OsRng);

    const MEMORY: u32 = 19 * 1024;
    const ITERATIONS: u32 = 2;
    const DEGREE_OF_PARALLELISM: u32 = 1;

    let params = ParamsBuilder::new()
        .m_cost(MEMORY)
        .t_cost(ITERATIONS)
        .p_cost(DEGREE_OF_PARALLELISM)
        .build()
        .unwrap();
    let argon2 = Argon2::new_with_secret(&*PEPPER, Algorithm::Argon2id, Version::V0x13, params).unwrap();

    let phc_format_hash = argon2.hash_password(password.value().as_bytes(), &salt).unwrap().to_string();

    PasswordHash(phc_format_hash)
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::common::password::{ParsePasswordError, Password, MAX_PASSWORD_LENGTH, MIN_PASSWORD_LENGTH};

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
        let pass = Password::from_str("0000000000");
        assert_eq!(pass, Err(ParsePasswordError::Unsafe));
    }
}