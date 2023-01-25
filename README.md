# Tree Structured Argument Parser

Configure large and complex applications written in Rust similar to [Hydra](https://hydra.cc/). In the library modus builder pattern and automatic hyperparameter checking is added. When compiled with `toml` flag the functionality is extended with a template and serialization support.

## Example

```rust
#[param]
#[derive(Debug)]
pub struct Main<const C: bool> {
    seed: u64,
    rev: String,
    experiment: Experiment<C>,
}

impl<const C: bool> ParamGuard for Main<C> {
    type Error = Error;

    fn check(&self) -> Result<(), Self::Error> {
    	match self.seed {
	    42 => Err(Error::InvalidArg("Seed 42 is not allowed".into())),
	    _ => Ok(())
	}
    }
}

fn main() -> Result<(), Error> {
    // populate configuration from main and amend arguments
    let main: Main<false> = Main::from_file("conf/main.toml")?
        .amend_args()?
        .try_into()?;

    // verify all hyper-parameters
    let mut main: Main<true> = main.build()?;

    // run experiment
    main.experiment.run();

    Ok(())
}

```

and configuration `main.toml`

```toml
seed = 100
rev = { cmd = "git rev-parse --short HEAD" }

[experiment]
variant = "TrainModel"
batch_size = 64
model = { from_file = { base_path = "conf/model", name = "bcresnet" }}

```
