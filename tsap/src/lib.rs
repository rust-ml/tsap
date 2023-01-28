extern crate tsap_macro;

pub use tsap_macro::param;

#[cfg(feature = "toml")]
pub mod toml_builder;
#[cfg(feature = "toml")]
pub mod templates;

mod error;

pub use error::{Result, Error};
#[cfg(feature = "toml")]
pub use toml_builder::{TomlBuilder, toml, serde, Path};

pub trait ParamGuard {
    type Error;

    fn check(&self) -> std::result::Result<(), Self::Error> {
        Ok(())
    }
}

/// Caller semantic to accept owned values and closures in builder pattern
pub struct Call<T>(Box<dyn FnOnce(T) -> T>);

impl<T> Call<T> {
    pub fn call(self, val: T) -> T {
        self.0(val)
    }
}

//impl<T: 'static> From<T> for Call<T> {
//    fn from(val: T) -> Call<T> {
//        Call(Box::new(move |_| val))
//    }
//}

impl<F: 'static, T> From<F> for Call<T> where F: FnOnce(T) -> T {
    fn from(val: F) -> Call<T> {
        Call(Box::new(val))
    }
}
//impl<F: 'static> From<F> for Call<usize> where F: FnOnce(usize) -> usize {
//    fn from(val: F) -> Call<usize> {
//        Call(Box::new(val))
//    }
//}
//
//impl<F: 'static> From<F> for Call<f64> where F: FnOnce(f64) -> f64 {
//    fn from(val: F) -> Call<f64> {
//        Call(Box::new(val))
//    }
//}

/// Caller semantic to accept owned values and closures in builder pattern
pub struct TryCall<T>(Box<dyn FnOnce(T) -> Result<T>>);

impl<T> TryCall<T> {
    pub fn call(self, val: T) -> Result<T> {
        self.0(val)
    }
}

impl<T: 'static> From<Result<T>> for TryCall<T> {
    fn from(val: Result<T>) -> TryCall<T> {
        TryCall(Box::new(move |_| val))
    }
}

impl<F: 'static> From<F> for TryCall<usize> where F: FnOnce(usize) -> Result<usize> {
    fn from(val: F) -> TryCall<usize> {
        TryCall(Box::new(val))
    }
}

impl<F: 'static> From<F> for TryCall<f64> where F: FnOnce(f64) -> Result<f64> {
    fn from(val: F) -> TryCall<f64> {
        TryCall(Box::new(val))
    }
}
