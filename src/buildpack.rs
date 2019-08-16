use failure::Error;
use serde_derive::Deserialize;
use std::{collections::HashMap, fs};
use toml::map::Map;

#[derive(Deserialize)]
pub struct Scriptless {
    pub buildpack: Buildpack,
}

impl Scriptless {
    pub fn load_toml() -> Result<Scriptless, Error> {
        let string = fs::read_to_string(std::env::current_dir()?.join("buildpack.toml"))?;

        Ok(toml::from_str(&string)?)
    }
}

#[derive(Deserialize)]
pub struct Buildpack {
    pub detect: Option<Detect>,
    pub build: Option<Build>,
}

#[derive(Deserialize)]
pub struct Detect {
    #[serde(default)]
    pub run: Vec<String>,
    pub requires: Option<Vec<String>>,
    pub provides: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct Build {
    #[serde(default)]
    pub run: Vec<String>,
    #[serde(default)]
    pub layers: Vec<Layer>,
    pub launch: Processes,
}

#[derive(Deserialize)]
pub struct Layer {
    pub id: String,
    #[serde(default)]
    pub cache: bool,
    #[serde(default)]
    pub launch: bool,
    #[serde(default)]
    pub build: bool,
    pub metadata: Map<String, toml::Value>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub profile: Vec<Profile>,
    pub run: Vec<String>,
}

#[derive(Deserialize)]
pub struct Profile {
    pub name: String,
    pub script: String,
}

#[derive(Deserialize)]
pub struct Processes {
    #[serde(default)]
    pub processes: Vec<Process>,
}

#[derive(Deserialize)]
pub struct Process {
    pub r#type: String,
    pub command: String,
}
