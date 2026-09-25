use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum MapError {
    BadMagic,
    UnsupportedVersion(u32),
    Decode(String),
    Encode(String),
    Invalid(String),
}

impl fmt::Display for MapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadMagic => write!(f, "bad map magic"),
            Self::UnsupportedVersion(version) => write!(f, "unsupported map version {version}"),
            Self::Decode(message) => write!(f, "map decode failed: {message}"),
            Self::Encode(message) => write!(f, "map encode failed: {message}"),
            Self::Invalid(message) => write!(f, "invalid map: {message}"),
        }
    }
}

impl std::error::Error for MapError {}
