use crate::common::token::{calc_entropy_bytes, Token};

const TOKEN_ENTROPY_BITS: usize = 120;

pub type OneTimeToken = Token<{calc_entropy_bytes(TOKEN_ENTROPY_BITS)}>;