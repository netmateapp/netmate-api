use std::{fmt::{self, Display}, str::FromStr};

use thiserror::Error;

use super::{character_count::calculate_character_cost, uuid::uuid4::Uuid4};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HandleId(Uuid4);

impl HandleId {
    pub fn gen() -> Self {
        HandleId(Uuid4::gen())
    }

    pub const fn of(value: Uuid4) -> Self {
        HandleId(value)
    }

    pub fn value(&self) -> Uuid4 {
        self.0
    }
}

impl Display for HandleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

const HANDLE_NAME_MAX_CHARACTER_COST: usize = 100;

pub struct HandleName(String);

impl HandleName {
    pub fn value(&self) -> &String {
        &self.0
    }
}

impl FromStr for HandleName {
    type Err = ParseHandleNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseHandleNameError::Empty);
        }

        // 長い文字列は文字コストを計算する前に弾く
        // CJK範囲の文字のみで構成された名義の最大長は、`HANDLE_NAME_MAX_CHARACTER_COST / 2` となる
        // 有効な名義のうち最もbyte数の多いものは、CJK統合拡張漢字B～F範囲の文字(4byte)のみで構成される
        // `HANDLE_NAME_MAX_CHARACTER_COST / 2 cost * 4 bytes` = `HANDLE_NAME_MAX_CHARACTER_COST * 2`
        if s.len() > HANDLE_NAME_MAX_CHARACTER_COST * 2 {
            return Err(ParseHandleNameError::CharacterCostOverflow);
        }

        if calculate_character_cost(s) > HANDLE_NAME_MAX_CHARACTER_COST {
            return Err(ParseHandleNameError::CharacterCostOverflow);
        }

        Ok(HandleName(String::from(s)))
    }
}

#[derive(Debug, Error)]
pub enum ParseHandleNameError {
    #[error("空の名義は許可されていません")]
    Empty,
    #[error("文字数が多すぎます")]
    CharacterCostOverflow,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HandleShareCount(u32);

impl HandleShareCount {
    pub const fn of(value: u32) -> Self {
        HandleShareCount(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}