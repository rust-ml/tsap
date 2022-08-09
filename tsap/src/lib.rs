extern crate tsap_macro;

pub use tsap_macro::param;

#[cfg(feature = "toml")]
pub mod toml_builder;
#[cfg(feature = "toml")]
pub mod templates;

mod error;

pub use error::{Result, Error};

pub trait ParamGuard {
    type Error;

    fn check(&self) -> std::result::Result<(), Self::Error> {
        Ok(())
    }
}

