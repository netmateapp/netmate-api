use std::str::FromStr;

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{de, Deserialize};
use thiserror::Error;

const ENTROPY_BITS_PER_CHAR: usize = 5;
const TOKEN_CHARSET: [char; 1 << ENTROPY_BITS_PER_CHAR] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5'];

pub struct Token<const ENTROPY_BYTES: usize>(String);

const OCTET: usize = 8;

// 特化トークン型の定義に使用可能なヘルパー関数
pub const fn calc_entropy_bytes(entropy_bits: usize) -> usize {
    entropy_bits / OCTET
}

impl<const ENTROPY_BYTES: usize> Token<ENTROPY_BYTES> {
    pub fn gen() -> Self {
        let mut rng = ChaCha20Rng::from_entropy();
        let mut random_bytes = [0u8; ENTROPY_BYTES];
        rng.fill_bytes(&mut random_bytes);

        let mut token = String::with_capacity(ENTROPY_BYTES * OCTET / ENTROPY_BITS_PER_CHAR);
        let mut bit_buffer: u32 = 0;
        let mut bit_count = 0;

        const MASK: u32 = 1 << ENTROPY_BITS_PER_CHAR - 1;

        for byte in random_bytes.iter() {
            bit_buffer |= (*byte as u32) << bit_count;
            bit_count += OCTET;

            while bit_count >= ENTROPY_BITS_PER_CHAR {
                let index = (bit_buffer & MASK) as usize;
                token.push(TOKEN_CHARSET[index]);
                bit_buffer >>= ENTROPY_BITS_PER_CHAR;
                bit_count -= ENTROPY_BITS_PER_CHAR;
            }
        }

        if bit_count > 0 {
            token.push(TOKEN_CHARSET[(bit_buffer & MASK) as usize]);
        }

        Self(token)
    }

    pub fn value(&self) -> &String {
        &self.0
    }
}

#[derive(Debug, Error)]
#[error("トークンの形式を満たしていません")]
pub struct ParseTokenError;

impl<const BYTES: usize> FromStr for Token<BYTES> {
    type Err = ParseTokenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let len = s.len();
        if len % 8 == 0 && len % 5 == 0 {
            if s.chars().all(|c| matches!(c, 'a'..='z' | '0'..='5')) {
                Ok(Self(String::from(s)))
            } else {
                Err(ParseTokenError)
            }
        } else {
            Err(ParseTokenError)
        }
    }
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