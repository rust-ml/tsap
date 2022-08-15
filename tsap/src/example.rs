use tsap::{param, ParamGuard};

#[param]
pub struct Param {
    pub ntrees: usize
}

impl ParamGuard for Param {
    type Error = tsap::Error;

    fn check(&self) -> Result<(), Self::Error> {
        if self.ntrees < 50 {
            return Err(tsap::Error::InvalidParam(
                format!("number trees >= 50, but is {}", self.ntrees)
            ));
        }

        Ok(())
    }
}

fn main() -> Result<(), tsap::Error> {
    let p = Param::from(toml::toml!(
        ntrees = 400
        rev = { cmd = "git status" }
    ));

    dbg!(&p.0.root);

    let p = Param::from_file("main.toml")?
        .ntrees(100)
        .amend_file("main.toml")?;

        //.amend(toml!(
        //    ntrees = 1000
        //))
        //.amend_args();


    let p = p.ntrees(100);
    dbg!(&p.get_ntrees());
    let p: Param = p.try_into().unwrap();

    Ok(())
}

//https://ferrous-systems.com/blog/testing-proc-macros/
//https://github.com/ferrous-systems/testing-proc-macros/blob/main/src/lib.rs
