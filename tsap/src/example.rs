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
    let p = ParamBuilder {
        val_ntrees: 10,
    };

    let p = p.ntrees(100);
    dbg!(&p.get_ntrees());
}

//https://ferrous-systems.com/blog/testing-proc-macros/
//https://github.com/ferrous-systems/testing-proc-macros/blob/main/src/lib.rs
