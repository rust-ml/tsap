use std::io::Read;
use std::path::Path;
use std::fs::File;
use std::env;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

use toml::Value;
use crate::{Result, Error, templates::Templates};

fn merge_use_second(a: toml::Value, b: toml::Value) -> Result<toml::Value> {
    match (a,b) {
        (Value::Float(_), Value::Float(b)) => Ok(Value::Float(b)),
        (Value::Integer(_), Value::Integer(b)) => Ok(Value::Integer(b)),
        (Value::String(_), Value::String(b)) => Ok(Value::String(b)),
        (Value::Boolean(_), Value::Boolean(b)) => Ok(Value::Boolean(b)),
        (Value::Datetime(_), Value::Datetime(b)) => Ok(Value::Datetime(b)),
        (Value::Array(_), Value::Array(arr)) => Ok(Value::Array(arr)),
        (Value::Table(mut t1), Value::Table(t2)) => {
            for (k,v) in t2 {
                t1.insert(k, v);
            }

            Ok(Value::Table(t1))
        },
        _ => Err(Error::MergeFailed)
    }
}

pub struct TomlBuilder {
    pub root: toml::Value,
    templates: Templates,
}

impl Default for TomlBuilder {
    fn default() -> TomlBuilder {
        TomlBuilder {
            root: toml::Value::Integer(0),
            templates: Templates::default(),
        }
    }
}

impl TryFrom<toml::Value> for TomlBuilder {
    type Error = Error;

    fn try_from(root: toml::Value) -> Result<TomlBuilder> {
        let mut builder = TomlBuilder {
            templates: Templates::default(),
            root,
        };

        builder.root = builder.templates.resolve(builder.root);

        Ok(builder)
    }
}

impl TryFrom<&str> for TomlBuilder {
    type Error = Error;

    fn try_from(content: &str) -> Result<TomlBuilder> {
        let root: toml::Value = content.parse()?;

        Self::try_from(root)
    }
}

impl TryFrom<String> for TomlBuilder {
    type Error = Error;

    fn try_from(content: String) -> Result<TomlBuilder> {
        content.as_str().try_into()
    }
}



impl TomlBuilder {
    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<TomlBuilder> {
        let mut f = File::open(path)?;

        let mut content = String::new();
        f.read_to_string(&mut content)?;
        
        content.try_into()
    }

    pub fn amend_args(mut self) -> Result<Self> {
        let args = env::args().into_iter().map(|x| {
            if x.matches('=').count() != 1 {
                return Err(Error::InvalidArg(x));
            }

            let elms = x.splitn(2, '=').into_iter().collect::<Vec<_>>();

            Ok((TomlPath::from_str(elms[0])?, elms[1].to_string()))
        }).collect::<Result<Vec<_>>>()?;

        for (key, val) in args {
            key.update(&mut self.root, val)?;
        }

        Ok(self)
    }

    pub fn amend_file<T: AsRef<Path>>(mut self, path: T) -> Result<Self> {
        let mut f = File::open(path)?;

        let mut content = String::new();
        f.read_to_string(&mut content)?;

        let root: toml::Value = content.parse()?;

        let root = self.templates.resolve(root);

        // merge both dictionaries
        self.root = merge_use_second(self.root, root)?;

        Ok(self)
    }

    //pub fn amend<T: TryInto<Value>>(mut self, val: T) -> Result<Self>
    //    where Error: From<<T as TryInto<Value>>::Error> {
    //    let root = val.try_into()?;
    //    let root = self.templates.resolve(root);

    //    // merge both dictionaries
    //    self.root = merge_use_second(self.root, root)?;

    //    Ok(self)
    //}
    
    pub fn root(self) -> toml::Value {
        self.root
    }
}

pub enum Operation {
    Update,
    Set,
    Delete
}

pub struct TomlPath(Vec<String>, Operation);

impl FromStr for TomlPath {
    type Err = Error;

    fn from_str(path: &str) -> Result<TomlPath> {
        let parsed_path = path.split('.').map(|x| x.to_string())
            .collect::<Vec<_>>();

        let operation = if path.starts_with('+') {
            Operation::Set
        } else if path.starts_with('~') {
            Operation::Delete
        } else {
            Operation::Update
        };

        Ok(TomlPath(parsed_path, operation))
    }
}

impl std::string::ToString for TomlPath {
    fn to_string(&self) -> String {
        self.0.join(".")
    }
}

impl TomlPath {
    pub fn update<T: Into<toml::Value>>(self, mut map: &mut toml::Value, value: T) -> Result<()> {
        let (last, rest) = self.0.split_last().unwrap();

        for key in rest {
            match map {
                toml::Value::Table(x) => {
                    if let Some(tmp) = x.get_mut(key) {
                        map = tmp;
                    } else {
                        return Err(Error::InvalidPath(self.to_string()));
                    }
                },
                _ => return Err(Error::InvalidPath(self.to_string()))
            }
        }

        let map = match map {
            toml::Value::Table(table) => table,
            _ => return Err(Error::InvalidPath(self.to_string()))
        };

        match (self.1, map.get(last).is_some()) {
            (Operation::Update, true) | (Operation::Set, _) => *map.get_mut(last).unwrap() = value.into(),
            (Operation::Update, false) => return Err(Error::KeyNotExists(last.to_string(), rest.join("."))),
            (Operation::Delete, true) => {map.remove(last).unwrap(); },
            (Operation::Delete, false) => return Err(Error::KeyNotExists(last.to_string(), rest.join("."))),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let _builder = TomlBuilder::default();
    }

    #[test]
    fn test_from_string() {
        let content = r#"seed = 10
[dataset]
path = 'data/cifar10/'
"#;
        let builder: TomlBuilder = content.try_into().unwrap();
        dbg!(&builder.root);
    }
}
