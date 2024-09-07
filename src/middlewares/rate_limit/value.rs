use thiserror::Error;

const MIN_NAMESPACE_LENGTH: usize = 3;
const MAX_NAMESPACE_LENGTH: usize = 9;

pub struct Namespace(&'static str);

impl Namespace {
    pub fn new(namespace: &'static str) -> Result<Self, InvalidNamespaceError> {
        if namespace.contains(':') {
            Err(InvalidNamespaceError::ContainsColon)
        } else if !namespace.is_ascii() {
            Err(InvalidNamespaceError::NotAscii)
        } else if namespace.len() < MIN_NAMESPACE_LENGTH {
            Err(InvalidNamespaceError::TooShort)
        } else if namespace.len() > MAX_NAMESPACE_LENGTH {
            Err(InvalidNamespaceError::TooLong)
        } else {
            Ok(Self(namespace))
        }
    }

    pub fn value(&self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Error)]
pub enum InvalidNamespaceError {
    #[error("コロンは許可されていません")]
    ContainsColon,
    #[error("ASCII文字列である必要があります")]
    NotAscii,
    #[error("{}文字以上である必要があります", MIN_NAMESPACE_LENGTH)]
    TooShort,
    #[error("{}文字以下である必要があります", MAX_NAMESPACE_LENGTH)]
    TooLong
}

pub struct Limit(u16);

impl Limit {
    pub fn new(limit: u16) -> Self {
        Self(limit)
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

pub enum Interval {
    Minutes(u32),
    Hours(u32)
}

impl Interval {
    pub fn as_secs(&self) -> u32 {
        match self {
            Self::Minutes(minutes) => *minutes * 60,
            Self::Hours(hours) => *hours as u32 * 60 * 60
        }
    }
}