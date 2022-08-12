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

/// Caller semantic to accept owned values and closures in builder pattern
pub struct Call<T>(Box<dyn Fn(T) -> T>);

impl<T> Call<T> {
    pub fn call(self, val: T) -> T {
        self.0(val)
    }
}

impl<T: Copy + 'static> From<T> for Call<T> {
    fn from(val: T) -> Call<T> {
        Call(Box::new(move |_| val))
    }
}

impl<F: 'static> From<F> for Call<usize> where F: Fn(usize) -> usize {
    fn from(val: F) -> Call<usize> {
        Call(Box::new(val))
    }
}

impl<F: 'static> From<F> for Call<f64> where F: Fn(f64) -> f64 {
    fn from(val: F) -> Call<f64> {
        Call(Box::new(val))
    }
}
