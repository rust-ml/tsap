#![feature(try_trait_v2)]

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

//#[param]
//pub enum Param<const C: bool> {
//    Blub(Param2<C, C>, u32),
//    Bla
//}
//#[param]
//#[derive(Debug)]
//pub struct SVClassifier<const C: bool> {
//    nu: f32
//}
//
//impl<const C: bool> ParamGuard for SVClassifier<C> {
//    type Error = tsap::Error;
//
//    fn check(&self) -> Result<(), Self::Error> {
//        Ok(())
//    }
//}
//
//impl Default for SVClassifier<true> {
//    fn default() -> Self {
//        SVClassifier {
//            nu: 0.1
//        }
//    }
//}
//
//#[param]
//#[derive(Debug)]
//pub enum Model<const C: bool> {
//    SVClassifier(SVClassifier<C>),
//}
//
//impl Default for Model<true> {
//    fn default() -> Self {
//        Model::SVClassifier(SVClassifier::default())
//    }
//}
//
//impl<const C: bool> ParamGuard for Model<C> {
//    type Error = tsap::Error;
//
//    fn check(&self) -> Result<(), Self::Error> {
//        Ok(())
//    }
//}

#[param]
#[derive(Debug)]
pub struct Param<const C: bool, T: Default> {
    seed: T,
    //model: Model<C>,
    //rev: String,
    //date: String,

    //model: ModelParam,
}

impl<T: Default> Default for Param<true, T> {
    fn default() -> Self {
        Param {
            seed: T::default()
            //rev: "Blub".into(),
            //date: "null".into(),
            //model: ModelParam::RandomForest { ntrees: 10 }
        }
    }
}

impl<const C: bool, T: Default> ParamGuard for Param<C, T> {
    type Error = tsap::Error;

    fn check(&self) -> Result<(), Self::Error> {
        Ok(())
    }
}
fn main() -> Result<(), tsap::Error> {
    let param = Param::default()
        .seed(|x| x+1)?;

    Ok(())
}

/*



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
}*/

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
