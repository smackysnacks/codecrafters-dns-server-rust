use std::error;
use std::fmt;

use bytes::TryGetError;

#[derive(Debug)]
pub enum DnsError {
    NotEnoughData(TryGetError),
}

impl fmt::Display for DnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DnsError::NotEnoughData(e) => write!(f, "not enough data to parse: {e}"),
        }
    }
}

impl error::Error for DnsError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            DnsError::NotEnoughData(e) => Some(e),
        }
    }
}

impl From<TryGetError> for DnsError {
    fn from(value: TryGetError) -> Self {
        Self::NotEnoughData(value)
    }
}

pub type Result<T> = std::result::Result<T, DnsError>;
