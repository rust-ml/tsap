# Tree Structured Argument Parser (WIP)

Configure large and complex applications written in Rust similar to [Hydra](https://hydra.cc/). In library modus a builder pattern and automatic hyperparameter check are added to structures. When compiled with `toml` flag the functionality is extended with templates and serialization support.

Compared to Hydra there are several key differences:

 - use TOML instead of YAML (because it is [hard](https://noyaml.com/) to [use](https://www.arp242.net/yaml-config.html))
 - hyper-parameter check is enforced during compile time
 - template engine allows different expansion, for example globbing, command line execution etc. 
 - sourcing from different files has no special construct and is treated as a template
 - integrated into Rust serde infrastructure 

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
you can then set subtrees of your configuration (use GRU model instead of BCResnet, change seed and batch size)

```bash
cargo run --release -- seed=50 experiment.model.from_file.name=gru experiment.batch_size=32
```
```
