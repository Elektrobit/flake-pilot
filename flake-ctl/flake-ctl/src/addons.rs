use std::{
    collections::HashMap,
    convert::Infallible,
    env::{self, split_paths},
    fmt::Display,
    fs,
    process::Command,
    str::FromStr,
};

use itertools::Itertools;

pub struct Addons {
    addons: HashMap<AddonType, Vec<Addon>>,
}

impl Addons {
    fn add(&mut self, addon: Addon) {
        self.addons.entry(addon.ty.clone().unwrap_or_default()).or_default().push(addon);
    }

    pub fn all(&self) -> Vec<Addon> {
        self.addons.values().flatten().cloned().collect()
    }
}

impl Display for Addons {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (kind, list) in &self.addons {
            writeln!(f, "\n-- {kind}s --")?;
            for Addon { name, description, .. } in list {
                writeln!(f, "{name}\t\t{}", description.as_ref().map(|s| s.as_str()).unwrap_or(""))?
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Addon {
    pub name: String,
    pub description: Option<String>,
    pub ty: Option<AddonType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AddonType {
    Engine,
    Tool,
    Other(String),
}

impl Default for AddonType {
    fn default() -> Self {
        Self::Tool
    }
}

impl Display for AddonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddonType::Engine => f.write_str("Engine"),
            AddonType::Tool => f.write_str("Tool"),
            AddonType::Other(s) => f.write_str(s),
        }
    }
}

impl FromStr for AddonType {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str().trim() {
            "engine" => Self::Engine,
            "tool" => Self::Tool,
            _ => Self::Other(s.to_owned()),
        })
    }
}

pub fn find_addons() -> Addons {
    let mut addons = Addons { addons: HashMap::new() };

    split_paths(&env::var_os("PATH").unwrap_or_default())
        .map(fs::read_dir)
        .filter_map(Result::ok)
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|x| x.file_name().to_str().map(str::to_owned))
        .filter_map(|name| name.strip_prefix("flake-ctl-").map(str::to_owned))
        .map(collect_info)
        .unique()
        .for_each(|a| addons.add(a));
    addons
}

fn collect_info(name: String) -> Addon {
    let info = Command::new(format!("flake-ctl-{name}"))
        .arg("about")
        .output()
        .ok()
        .map(|o| o.stdout)
        .map(|o| String::from_utf8_lossy(&o).to_string());
    match info {
        Some(info) => {
            let mut info = info.trim().split(';');
            let description = info.next().map(str::to_owned);
            let ty = info.next().and_then(|x| x.parse().ok());
            Addon { name, description, ty }
        }
        None => Addon { name, description: None, ty: None },
    }
}
