use std::error;
use std::fmt;

#[derive(Debug)]
pub enum DnsError {
    Parse(String),
}

impl fmt::Display for DnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            DnsError::Parse(ref s) => write!(f, "failed to parse DNS packet: {s}"),
        }
    }
}

impl error::Error for DnsError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            DnsError::Parse(_) => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, DnsError>;
