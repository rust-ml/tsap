#![feature(try_trait_v2)]

use tsap::{param, ParamGuard};

#[param]
#[derive(Debug)]
pub enum ModelParam {
    RandomForest {
        ntrees: usize
    },
    SVClassifier {
        nu: f32
    }
}

impl ParamGuard for ModelParam {
    type Error = tsap::Error;
}

#[param]
#[derive(Debug)]
pub struct Param {
    seed: usize,
    rev: String,
    date: String,

    model: ModelParam,
}

impl Default for Param {
    fn default() -> Self {
        Param {
            seed: 0,
            rev: "Blub".into(),
            model: ModelParam::RandomForest { ntrees: 10 }
        }
    }
}

impl ParamGuard for Param {
    type Error = tsap::Error;

    fn check(&self) -> Result<(), Self::Error> {
        if self.rev.len() != 7 {
            return Err(tsap::Error::InvalidParam(
                format!("short revision should have length 7, but is {}", self.rev)
            ));
        }

        Ok(())
    }
}

fn main() -> Result<(), tsap::Error> {
    let p = Param::default()
        .seed(100)?
        .seed(50)
        .model(ModelParam::SVClassifier { nu: 100.0 })?;

    Ok(())
}

/*fn main() -> Result<(), tsap::Error> {
    let p = Param::from(toml::toml!(
        rev = { cmd = "git rev-parse --short HEAD" }
        date = { cmd = "date '+%d-%m-%Y %H:%M:%S'" }
        seed = 100

        [model]
        from_file = { base_path = "config/models/", default = "randomforest" }
    ));

    dbg!(&p.0.root);

    let p = Param::from_file("config/main.toml")?
        .seed(50)
        .seed(|x| x+3)
        .seed(|x| x+3)
        .amend_file("config/main.toml")?;

    dbg!(&p.get_seed());

    let p: Param = p.try_into()?;

    dbg!(&p);

    Ok(())
}*/

//https://ferrous-systems.com/blog/testing-proc-macros/
//https://github.com/ferrous-systems/testing-proc-macros/blob/main/src/lib.rs
