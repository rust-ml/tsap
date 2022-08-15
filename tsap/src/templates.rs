use std::io::Read;
use std::fs::File;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use serde::Deserialize;
use toml::{value::Table, Value};

pub type DynTemplate = Box<dyn Template>;

pub trait Template {
    fn resolve(&mut self, key: String, map: Table, field: toml::Value) -> Value;
}

pub struct Templates(HashMap<String, Box<dyn Template>>);

impl Default for Templates {
    fn default() -> Templates {
        Templates(HashMap::from([
            ("from_file".to_string(), Box::new(FromFile::default()) as DynTemplate),
            ("cmd".to_string(), Box::new(RunCommand::default()) as DynTemplate),
        ]))
    }
}

impl Templates {
    pub fn resolve(&mut self, root: toml::Value) -> toml::Value {
        let root = match root {
            toml::Value::Table(mut map) => {
                for (name, resolver) in self.0.iter_mut() {
                    if let Some(value) = map.remove(name) {
                        match resolver.resolve("".into(), map, value) {
                            Value::Table(new_map) => map = new_map,
                            x => return x
                        }
                    }
                }

                let map = map.into_iter()
                    .map(|(k,v)| (k, self.resolve(v)))
                    .collect();

                toml::Value::Table(map)
            },
            x => x,
        };

        root
    }
}

#[derive(Default)]
pub struct RunCommand;

impl Template for RunCommand {
    fn resolve(&mut self, key: String, _map: Table, field: toml::Value) -> Value {
        let cmd = match field {
            Value::String(cmd) => cmd,
            _ => panic!("Not a string as command")
        };

        let cmd = cmd.split(" ").collect::<Vec<_>>();

        let output = Command::new(&cmd[0])
            .args(&cmd[1..])
            .output()
            .unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();

        Value::String(stdout)
    }
}

#[derive(Default)]
pub struct FromFile {
    base_path: HashMap<String, PathBuf>,
}

impl Template for FromFile {
    fn resolve(&mut self, key: String, _map: Table, field: toml::Value) -> Value {
        let field: FromFileField = field.try_into().unwrap();

        let mut f = File::open(field.base_path.join(&format!("{}.toml", field.default))).unwrap();
        let mut content = String::new();
        f.read_to_string(&mut content).unwrap();

        self.base_path.insert(key, field.base_path).unwrap();

        // override current node with content of file
        content.parse::<toml::Value>().unwrap()
    }
}

#[derive(Deserialize)]
struct FromFileField {
    base_path: PathBuf,
    default: String
}
