use std::env::var;
use std::fmt::Display;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;

use serde::Deserialize;

#[derive(Debug)]
pub enum ConfigError {
    IOError(std::io::Error),
    ParseError(toml::de::Error),
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
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    repo: Repo,
    shortcut: Shortcut,

    #[allow(dead_code)]
    colors: Color,
}

#[derive(Deserialize, Debug, Clone)]
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
        self.attempt_set_default()?;
        let content = self.load_config()?;
        Config::from_str(&content)
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

    fn attempt_set_default(&self) -> Result<(), io::Error> {
        match File::open(&self.path) {
            Ok(_) => Ok(()),
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => self.create_default(),
                _ => Err(err),
            },
        }
    }

    fn create_default(&self) -> Result<(), io::Error> {
        let mut f = File::create(&self.path)?;
        f.write_all(DEFAULT_CONFIG_CONTENT.as_bytes())?;
        Ok(())
    }

    fn load_config(&self) -> Result<String, ConfigError> {
        let mut f = File::open(&self.path)?;
        let mut buf = vec![];
        f.read_to_end(&mut buf)?;
        Ok(String::from_utf8(buf)?)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Repo {
    path: String,
}

const DEFAULT_CONFIG_CONTENT: &'static str = r###"
[repo]
path = "RustyBrain"

[shortcut]
find = "<Control><Shift>f"
insert = "<Control>i"
quit = "<Meta>q"

[colors]
primary = "#546E7A"
primary_text = "#FAFAFA"
primary_dark = "#29434e"
primary_light = "#819ca9"

secondary = "#B2EBF2"
secondary_light = "#e5ffff"
secondary_dark = "#81b9bf"
secondary_text = "#000000"

foreground = "#546E7A"
background = "#FAFAFA"

base00 = "#FAFAFA"
base01 = "#90A4AE"
base02 = "#78909C"
base03 = "#546E7A"
yellow = "#F57F17"
yellow1 = "#F9A725"
brown = "#4E342E"
brown1 = "#6D4C41"
orange = "#D84315"
orange1 = "#FF5722"
red = "#D50000"
red1 = "#FF1744"
pink = "#F8BBD0"
pink1 = "#EC407A"
purple = "#7E57C2"
purple1 = "#B388FF"
blue = "#42A5F5"
blue1 = "#1E88E5"
indigo = "#5C6BC0"
indigo1 = "#9FA8DA"
cyan = "#0097A7"
cyan1 = "#00B8D4"
teal = "#26A69A"
teal1 = "#00897B"
green = "#66BB6A"
green1 = "#558B2F"
"###;

#[derive(Deserialize, Debug, Clone)]
pub struct Color {
    #[allow(dead_code)]
    primary: String,
    #[allow(dead_code)]
    primary_light: String,
    #[allow(dead_code)]
    primary_dark: String,
    #[allow(dead_code)]
    primary_text: String,

    #[allow(dead_code)]
    secondary: String,
    #[allow(dead_code)]
    secondary_light: String,
    #[allow(dead_code)]
    secondary_dark: String,
    #[allow(dead_code)]
    secondary_text: String,

    #[allow(dead_code)]
    foreground: String,
    #[allow(dead_code)]
    background: String,

    #[allow(dead_code)]
    yellow: String,
    #[allow(dead_code)]
    brown: String,
    #[allow(dead_code)]
    orange: String,
    #[allow(dead_code)]
    red: String,
    #[allow(dead_code)]
    pink: String,
    #[allow(dead_code)]
    purple: String,
    #[allow(dead_code)]
    blue: String,
    #[allow(dead_code)]
    indigo: String,
    #[allow(dead_code)]
    cyan: String,
    #[allow(dead_code)]
    teal: String,
    #[allow(dead_code)]
    green: String,
}

#[cfg(test)]
mod tests {
    use super::ConfigLoader;

    #[test]
    fn test_default_config_loader() {
        let loader = ConfigLoader::new();
        let config = loader.load().unwrap();
        assert_eq!(config.colors.red, "#D50000");
    }
}
