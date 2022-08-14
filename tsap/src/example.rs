extern crate tsap;

use std::convert::TryInto;
use tsap::{param, ParamGuard};
use thiserror::Error;

#[param]
struct Param {
    ntrees: usize
}

impl ParamGuard for Param {
    type Error = ParamError;

    fn check(&self) -> Result<(), Self::Error> {
        if self.ntrees < 50 {
            Err(ParamError::InvalidNTrees)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Error)]
enum ParamError {
    #[error("invalid number of trees")]
    InvalidNTrees,
    #[error("internal error by tsap")]
    Tsap(#[from] tsap::Error),
}

fn main() {
    let p = Param::from_file("config/main.toml").unwrap()
        .ntrees(100);
        //.amend_file("config/main.toml")
        //.amend(toml!(
        //    ntrees = 1000
        //))
        //.amend_args();


    let p = p.ntrees(100).ntrees(40);
    dbg!(&p.get_ntrees());
    let p: Param = p.try_into().unwrap();
}

//https://ferrous-systems.com/blog/testing-proc-macros/
//https://github.com/ferrous-systems/testing-proc-macros/blob/main/src/lib.rs
