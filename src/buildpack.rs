use failure::Error;
use serde_derive::Deserialize;
use std::{
    collections::HashMap,
    fs,
    ops::Deref,
    path::PathBuf,
    process::{Child, Command, Stdio},
};
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
    pub run: Run,
    pub requires: Option<Vec<String>>,
    pub provides: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct Build {
    #[serde(default)]
    pub run: Run,
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
    pub run: Run,
}

impl Layer {
    pub fn should_rebuild(&self, cache: bool, metadata: &Map<String, toml::Value>) -> bool {
        if cache && &self.metadata == metadata {
            true
        } else {
            false
        }
    }
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

#[derive(Deserialize)]
pub struct Run(Vec<String>);

impl Deref for Run {
    type Target = Vec<String>;

    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}

impl Default for Run {
    fn default() -> Self {
        Self { 0: Vec::new() }
    }
}

impl Run {
    pub fn execute(&self, cli_args: &[&PathBuf]) -> Result<Child, Error> {
        let tmpdir = tempdir::TempDir::new("run")?;
        let run_script_path = tmpdir.path().join("run.sh");
        fs::write(&run_script_path, self.0.join("\n"))?;

        let mut args = Vec::new();
        args.push(&run_script_path);
        args.extend_from_slice(cli_args);

        let cmd = Command::new("bash")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        Ok(cmd)
    }
}
