use std::env::var;
use std::fmt::Display;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ConfigError {
    IOError(std::io::Error),
    ParseError(toml::de::Error),
    SaveError(toml::ser::Error),
    CodecError(FromUtf8Error),
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError::IOError(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::ParseError(err)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(e: toml::ser::Error) -> Self {
        ConfigError::SaveError(e)
    }
}

impl From<FromUtf8Error> for ConfigError {
    fn from(err: FromUtf8Error) -> Self {
        ConfigError::CodecError(err)
    }
}

impl From<ConfigError> for String {
    fn from(c: ConfigError) -> Self {
        format!("{}", c)
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IOError(e) => e.fmt(f),
            ConfigError::ParseError(e) => e.fmt(f),
            ConfigError::CodecError(e) => e.fmt(f),
            ConfigError::SaveError(e) => e.fmt(f),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Config {
    repo: Repo,
    shortcut: Shortcut,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Shortcut {
    find: String,
    insert: String,
    quit: String,
}

impl Config {
    pub fn from_str(s: &str) -> Result<Self, ConfigError> {
        let config = toml::from_str(s)?;
        Ok(config)
    }

    pub fn repo_path(&self) -> &str {
        &self.repo.path
    }

    pub fn shortcut(&self) -> &Shortcut {
        &self.shortcut
    }

    pub fn is_default(&self) -> bool {
        self.repo.path == DEFAULT_REPO_PATH
    }
}

impl Shortcut {
    pub fn find(&self) -> &str {
        &self.find
    }

    pub fn insert(&self) -> &str {
        &self.insert
    }

    pub fn quit(&self) -> &str {
        &self.quit
    }
}

impl Default for Shortcut {
    fn default() -> Self {
        Self {
            find: "<Control><Shift>f".to_string(),
            insert: "<Control>i".to_string(),
            quit: "<Meta>q".to_string(),
        }
    }
}

pub struct ConfigLoader {
    #[allow(dead_code)]
    home: PathBuf,
    dir: PathBuf,
    path: PathBuf,
}

impl ConfigLoader {
    pub fn new() -> Self {
        let raw_home = var("HOME").unwrap();
        let home = Path::new(&raw_home).to_path_buf();
        let dir = home.join(".rustybrain");
        let path = dir.join("config.toml");
        ConfigLoader { home, dir, path }
    }

    pub fn load(&self) -> Result<Config, ConfigError> {
        self.create_dir()?;
        if Self::is_exists(&self.path) {
            let content = self.load_config()?;
            Config::from_str(&content)
        } else {
            Ok(Config::default())
        }
    }

    fn create_dir(&self) -> Result<(), io::Error> {
        if Self::is_exists(&self.dir) {
            return Ok(());
        }
        fs::create_dir(&self.dir)?;
        Ok(())
    }

    fn is_exists(path: &Path) -> bool {
        match fs::metadata(path) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn save(&self, c: &Config) -> Result<(), ConfigError> {
        let content = toml::to_string(c)?;
        let mut f = File::create(&self.path)?;
        f.write_all(content.as_bytes())?;
        Ok(())
    }

    fn load_config(&self) -> Result<String, ConfigError> {
        let mut f = File::open(&self.path)?;
        let mut buf = vec![];
        f.read_to_end(&mut buf)?;
        Ok(String::from_utf8(buf)?)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Repo {
    path: String,
}

impl Default for Repo {
    fn default() -> Self {
        Self {
            path: DEFAULT_REPO_PATH.to_string(),
        }
    }
}

const DEFAULT_REPO_PATH: &'static str = "__default";
