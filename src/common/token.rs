use std::{fmt::{self, Display, Formatter}, str::FromStr};

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use redis::{FromRedisValue, RedisError, RedisResult};
use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{de, Deserialize};
use thiserror::Error;

const ENTROPY_BITS_PER_CHAR: usize = 6;

const BASE64_URL: [char; 1 << ENTROPY_BITS_PER_CHAR] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', // A-Z
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', // a-z
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', // 0-9
    '-', '_', // - と _
];


#[derive(Debug, Clone, PartialEq)]
pub struct Token<const ENTROPY_BYTES: usize>(String);

impl<const ENTROPY_BYTES: usize> Display for Token<ENTROPY_BYTES> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

const OCTET: usize = 8;

// 特化トークン型の定義に使用可能なヘルパー関数
pub const fn calc_entropy_bytes(entropy_bits: usize) -> usize {
    entropy_bits / OCTET
}

const fn calc_token_length(entropy_bytes: usize) -> usize {
    entropy_bytes * OCTET / ENTROPY_BITS_PER_CHAR
}

impl<const ENTROPY_BYTES: usize> Token<ENTROPY_BYTES> {
    #[cfg(debug_assertions)]
    pub fn new_unchecked(s: &str) -> Self {
        Self(String::from(s))
    }

    pub fn gen() -> Self {
        let mut rng = ChaCha20Rng::from_entropy();
        let mut random_bytes = [0u8; ENTROPY_BYTES];
        rng.fill_bytes(&mut random_bytes);

        let mut token = String::with_capacity(ENTROPY_BYTES * OCTET / ENTROPY_BITS_PER_CHAR);
        let mut bit_buffer: u16 = 0;
        let mut bit_count = 0;

        const MASK: u16 = (1 << ENTROPY_BITS_PER_CHAR) - 1;

        for byte in random_bytes.iter() {
            bit_buffer |= (*byte as u16) << bit_count;
            bit_count += OCTET;

            while bit_count >= ENTROPY_BITS_PER_CHAR {
                let index = (bit_buffer & MASK) as usize;
                token.push(BASE64_URL[index]);
                bit_buffer >>= ENTROPY_BITS_PER_CHAR;
                bit_count -= ENTROPY_BITS_PER_CHAR;
            }
        }

        if bit_count > 0 {
            token.push(BASE64_URL[(bit_buffer & MASK) as usize]);
        }

        Self(token)
    }

    pub fn value(&self) -> &String {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum ParseTokenError {
    #[error("文字列長が正しくありません")]
    InvalidLength,
    #[error("無効な文字が使用されています")]
    InvalidCharset,
}

impl<const ENTROPY_BYTES: usize> FromStr for Token<ENTROPY_BYTES> {
    type Err = ParseTokenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == calc_token_length(ENTROPY_BYTES) {
            // allは短絡評価であるため、一度`false`が返るとその時点で終了する
            if s.chars().all(is_valid_char) {
                Ok(Self(String::from(s)))
            } else {
                Err(ParseTokenError::InvalidCharset)
            }
        } else {
            Err(ParseTokenError::InvalidLength)
        }
    }
}

fn is_valid_char(c: char) -> bool {
    matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_')
}

impl<'de, const BYTES: usize> Deserialize<'de> for Token<BYTES> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Token::from_str(s).map_err(de::Error::custom)
    }
}

impl<const BYTES: usize> SerializeValue for Token<BYTES> {
    fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
        self.0.serialize(typ, writer)
    }
}

impl<const BYTES: usize> FromCqlVal<Option<CqlValue>> for Token<BYTES> {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        String::from_cql(cql_val).and_then(|v| Token::from_str(v.as_str()).map_err(|_| FromCqlValError::BadVal))
    }
}

impl<const BYTES: usize> FromRedisValue for Token<BYTES> {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        String::from_redis_value(v)
            .and_then(|v| Token::from_str(&v).map_err(|_| RedisError::from((redis::ErrorKind::TypeError, "トークンの形式を満たしていません"))))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::common::token::{calc_entropy_bytes, Token, ENTROPY_BITS_PER_CHAR};

    #[test]
    fn calc_bytes() {
        assert_eq!(calc_entropy_bytes(120), 120 / 8);
    }

    #[test]
    fn gen_token() {
        let token = Token::<{calc_entropy_bytes(120)}>::gen();
        assert_eq!(token.value().len(), 120 / ENTROPY_BITS_PER_CHAR);
    }

    #[test]
    fn valid_token() {
        assert!(Token::<{calc_entropy_bytes(120)}>::from_str("a02jA_rm-2hSixu2Bqv0").is_ok());
    }

    #[test]
    fn invalid_length_token() {
        assert!(Token::<{calc_entropy_bytes(120)}>::from_str("a02jA_rm-2hSixu2Bqv").is_err());
    }

    #[test]
    fn invalid_characters_token() {
        assert!(Token::<{calc_entropy_bytes(120)}>::from_str("a02jform52hzifu2kqod0ex_").is_err());
    }
}