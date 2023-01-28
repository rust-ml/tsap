use thiserror::Error;
use std::convert::Infallible;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[cfg(feature = "toml")]
    #[error("could not parse configuration")]
    TomlParse(#[from] toml::de::Error),
    #[cfg(feature = "toml")]
    #[error("invalid key path")]
    InvalidPath(String),
    #[error("invalid argument")]
    InvalidArg(String),
    #[error("merging dictionaries failed")]
    MergeFailed,
    #[error("key {0} does not exist in {1}")]
    KeyNotExists(String, String),
    #[error("parameter violates constrain: {0}")]
    InvalidParam(String),
    #[error("input/output error")]
    Io {
        #[from]
        source: std::io::Error,
    },
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}
