use std::io::Read;
use std::fs::File;
use std::collections::HashMap;
use std::path::PathBuf;
use serde::Deserialize;

pub type DynTemplate = Box<dyn Template>;

pub trait Template {
    fn resolve(&mut self, key: String, map: toml::Value, field: toml::Value) -> toml::Value;
}

pub struct Templates(HashMap<String, Box<dyn Template>>);

impl Default for Templates {
    fn default() -> Templates {
        Templates(HashMap::from([
            ("from_file".to_string(), Box::new(FromFile::default()) as DynTemplate),
        ]))
    }
}

impl Templates {
    pub fn resolve(&mut self, root: toml::Value) -> toml::Value {
        root
    }
}

#[derive(Default)]
pub struct FromFile {
    base_path: HashMap<String, PathBuf>,
}

impl Template for FromFile {
    fn resolve(&mut self, key: String, _map: toml::Value, field: toml::Value) -> toml::Value {
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
