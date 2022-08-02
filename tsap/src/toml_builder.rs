use std::io::Read;
use std::path;
use std::fs::File;
use std::env;
use std::mem;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

use toml::Value;
use crate::{Result, Error, templates::Templates};

fn merge(mut root: Value, action: Action) -> (Value, Vec<Action>) {
    // first iterate through root until we are after our path base
    let mut local = &mut root;
    let paths = action.path().0.clone();
    let num = paths.len() - 1;

    for path in &paths[..=num-1] {
        let local = match local {
            Value::Table(ref mut t) if t.contains_key(path) =>
                local = t.get_mut(path).unwrap(),
            _ => return (root, vec![action])
        };
    }

    let local = match local {
        Value::Table(ref mut t) if t.contains_key(&paths[num]) => t,
        _ => return (root, vec![action])
    };

    // now that we are at the base, use recursive merging
    let deferred = match action {
        Action::Delete(_) => { local.remove(&paths[num]).unwrap(); Vec::new() },
        Action::Set(_, val) => 
            merge_use_second(local.get_mut(&paths[num]).unwrap(), val, Mode::Set),
        Action::Modify(_, val) => 
            merge_use_second(local.get_mut(&paths[num]).unwrap(), val, Mode::Modify),
    };

    // add base path again if something is deferred
    let deferred = deferred.into_iter().map(|mut p| {
        let mut new = paths.clone();
        new.append(&mut p.mut_path().0);

        p.mut_path().0 = new;

        p
    }).collect();

    (root, deferred)
}

fn merge_use_second(a: &mut Value, b: Value, mode: Mode) -> Vec<Action> {
    match (a,b) {
        (Value::Float(ref mut a), Value::Float(b)) => { *a = b; vec![] },
        (Value::Integer(ref mut a), Value::Integer(b)) => { *a = b; vec![] },
        (Value::String(ref mut a), Value::String(b)) => { *a = b; vec![] },
        (Value::Boolean(ref mut a), Value::Boolean(b)) => { *a = b; vec![] },
        (Value::Datetime(ref mut a), Value::Datetime(b)) => { *a = b; vec![] },
        (Value::Array(ref mut a), Value::Array(b)) => { *a = b; vec![] },
        (Value::Table(ref mut t1), Value::Table(t2)) => {
            let mut deferred = Vec::new();

            // iterate through both tables
            // in set mode, we overwrite the entry, in modify mode we deferr until expansion
            for (k,v) in t2 {
                if let Some(ref mut a) = t1.get_mut(&k) {
                    let res = merge_use_second(a, v, mode)
                        .into_iter().map(|mut p| {
                            p.mut_path().0.push(k.clone().into());

                            p
                        });

                    deferred.extend(res);
                } else {
                    if mode == Mode::Set {
                        t1.insert(k, v);
                    } else {
                        return vec![Action::new(vec![k], v, mode)];
                    }
                }
            }

            deferred
        },
        (_, b) => vec![Action::new(vec![], b, mode)],
    }
}

#[derive(Debug)]
pub struct Path(Vec<String>);

impl FromStr for Path {
    type Err = Error;

    fn from_str(path: &str) -> Result<Path> {
        let parsed_path = path.split('.').map(|x| x.to_string())
            .collect::<Vec<_>>();

        Ok(Path(parsed_path))
    }
}

impl std::string::ToString for Path {
    fn to_string(&self) -> String {
        self.0.join(".")
    }
}

#[derive(Debug)]
pub enum Action {
    Modify(Path, Value),
    Set(Path, Value),
    Delete(Path),
}

impl Action {
    pub fn path(&self) -> &Path {
        match self {
            Action::Modify(p, _) => p,
            Action::Set(p, _) => p,
            Action::Delete(p) => p,
        }
    }

    pub fn mut_path(&mut self) -> &mut Path {
        match self {
            Action::Modify(p, _) => p,
            Action::Set(p, _) => p,
            Action::Delete(p) => p,
        }
    }

    pub fn new(p: Vec<String>, val: Value, mode: Mode) -> Self {
        let p = Path(p);

        match mode {
            Mode::Delete => Action::Delete(p),
            Mode::Modify => Action::Modify(p, val),
            Mode::Set => Action::Set(p, val),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Modify,
    Set,
    Delete,
}

pub struct TomlBuilder {
    pub root: toml::Value,
    templates: Templates,
    actions: Vec<Action>,
}

impl Default for TomlBuilder {
    fn default() -> TomlBuilder {
        TomlBuilder {
            root: toml::Value::Integer(0),
            templates: Templates::default(),
            actions: Vec::new()
        }
    }
}

impl TryFrom<toml::Value> for TomlBuilder {
    type Error = Error;

    fn try_from(root: toml::Value) -> Result<TomlBuilder> {
        let mut builder = TomlBuilder {
            templates: Templates::default(),
            actions: Vec::new(),
            root,
        };

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
    pub fn from_file<T: AsRef<path::Path>>(path: T) -> Result<TomlBuilder> {
        let mut f = File::open(path)?;

        let mut content = String::new();
        f.read_to_string(&mut content)?;
        
        content.try_into()
    }

    pub fn amend_args(mut self) -> Result<Self> {
        let mut mode = Mode::Modify;
            
        for arg in env::args().skip(1) {
            match arg.as_str() {
                "-a" => { mode = Mode::Set; continue },
                "-m" => { mode = Mode::Modify; continue },
                "-d" => { mode = Mode::Delete; continue },
                _ => {}
            }

            let elm = if mode != Mode::Delete {
                if arg.matches('=').count() != 1 {
                    return Err(Error::InvalidArg(arg));
                }

                let elms = arg.splitn(2, '=').into_iter().collect::<Vec<_>>();
                let path = Path::from_str(&elms[0]).unwrap();
                let value = if let Ok(val) = elms[1].parse::<i64>() {
                    Value::Integer(val)
                } else {
                    Value::String(elms[1].to_string())
                };

                match mode {
                    Mode::Modify => Action::Modify(path, value),
                    Mode::Set => Action::Set(path, value),
                    _ => unreachable!()
                }
            } else {
                let path = Path::from_str(&arg).unwrap();
                Action::Delete(path)
            };

            self.actions.push(elm);
        }

        Ok(self)
    }

    pub fn resolve_templates(&mut self) -> bool {
        let (root, any_changed) = self.templates.resolve(mem::replace(&mut self.root, Value::Integer(0)));
        self.root = root;

        any_changed
    }

    pub fn apply_actions(&mut self) -> bool {
        let mut deferred = Vec::new();

        // actions are applied in the same order as they came in
        for action in self.actions.drain(..) {
            let (root, mut tmp) = merge(mem::replace(&mut self.root, Value::Integer(0)), action);
            self.root = root;
            deferred.append(&mut tmp);
        }

        self.actions.append(&mut deferred);

        !self.actions.is_empty()
    }

    pub fn apply(&mut self) -> Result<()> {
        let mut any_resolved = true;

        // apply alternating actions and expand templates
        loop {
            let deferred_actions = self.apply_actions();
            if !any_resolved {
                if deferred_actions {
                    // we are stuck, there are pending actions but no templates expansion left
                    return Err(Error::MergeFailed);
                } else {
                    // done
                    return Ok(());
                }
            }

            any_resolved = self.resolve_templates()
        }
    }

    //pub fn amend_file<T: AsRef<Path>>(mut self, path: T) -> Result<Self> {
    //    let mut f = File::open(path)?;
    //    let mut content = String::new();
    //    f.read_to_string(&mut content)?;
    //    let root: toml::Value = content.parse()?;
    //    let root = self.templates.resolve(root);
    //    // merge both dictionaries
    //    self.root = merge_use_second(self.root, root)?;
    //    Ok(self)
    //}

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
