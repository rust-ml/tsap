extern crate tsap;

use tsap::{param, ParamGuard};

#[param]
struct Param {
    ntrees: usize
}

impl ParamGuard for Param {
    type Error = ParamError;
}

enum ParamError {
    InvalidNTrees,
}

fn main() {
}

//https://ferrous-systems.com/blog/testing-proc-macros/
//https://github.com/ferrous-systems/testing-proc-macros/blob/main/src/lib.rs
