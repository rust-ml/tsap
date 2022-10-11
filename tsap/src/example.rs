//#![feature(try_trait_v2)]

use tsap::{param, ParamGuard, Call};

/*
#[param]
#[derive(Debug)]
pub struct SVClassifier {
    nu: f32
}

impl ParamGuard for SVClassifier {
    type Error = tsap::Error;

    fn check(&self) -> Result<(), Self::Error> {
        if self.nu < 0.0 {
            return Err(tsap::Error::InvalidParam(
                format!("SV classifier regularization should be positive, but {}", self.nu)
            ));
        }

        Ok(())
    }
}

impl Default for SVClassifier {
    fn default() -> Self {
        SVClassifier {
            nu: 100.0,
        }
    }
}

#[param]
#[derive(Debug)]
pub enum ModelParam {
    RandomForest {
        ntrees: usize
    },
    SVClassifier(SVClassifier),
}

impl From<SVClassifier> for Call<ModelParam> {
    fn from(val: SVClassifier) -> Self {
        Call::from(ModelParam::SVClassifier(val))
    }
}

impl ParamGuard for ModelParam {
    type Error = tsap::Error;
}
*/

#[param]
#[derive(Debug)]
pub struct Param<const C: bool, T> {
    seed: usize,
    blub: T,
    //rev: String,
    //date: String,

    //model: ModelParam,
}

impl Default for Param<true> {
    fn default() -> Self {
        Param {
            seed: 0,
            //rev: "Blub".into(),
            //date: "null".into(),
            //model: ModelParam::RandomForest { ntrees: 10 }
        }
    }
}

impl<const C: bool> ParamGuard for Param<C> {
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
        .seed(100)
        .build()?
        .seed(|x| x+1)
        //.try_seed(|x| Ok(x + 1000))
        .model(|x| x.svclassifier(|x| x.nu(10.0)))
        // do we really need a Box<TryInto<SVClassifierParam>> here? Distinction between Param and
        // ParamBuilder not easy to make, but how plays this together with trait bound on return
        // value
        //.try_model(|x| x.try_svclassifier(|x| x.nu(10.0)))
        //.model(|obj| obj.nu(1000.0).build()?)
        //.model(ModelParam::SVClassifier { nu: 100.0 })
        .build()?;

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
