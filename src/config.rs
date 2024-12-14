use crate::aliases::{Alias, Aliases};
use crate::error::{Result, ShadowError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "Config::current_version")]
    version: u32,
    #[serde(default)]
    settings: Settings,
    #[serde(default)]
    #[serde(skip_serializing_if = "Aliases::is_empty")]
    aliases: Aliases,
}

impl Config {
    const CURRENT_VERSION: u32 = 1;

    fn current_version() -> u32 {
        Self::CURRENT_VERSION
    }

    pub fn new() -> Result<Self> {
        let config = Config {
            version: Self::CURRENT_VERSION,
            settings: Settings::default(),
            aliases: Aliases::default(),
        };
        config.save()?;
        Ok(config)
    }

    pub fn load() -> Result<Self> {
        if Self::config_path().exists() {
            let contents = std::fs::read_to_string(Self::config_path())?;
            let mut config: Config =
                toml::from_str(&contents).map_err(|e| ShadowError::ConfigError(e.to_string()))?;

            if config.version < Self::CURRENT_VERSION {
                config = config.migrate()?;
            }

            Ok(config)
        } else {
            Self::new()
        }
    }

    fn migrate(self) -> Result<Self> {
        match self.version {
            _ => Ok(self),
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = Self::config_path().parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents =
            toml::to_string_pretty(self).map_err(|e| ShadowError::ConfigError(e.to_string()))?;

        std::fs::write(Self::config_path(), contents)
            .map_err(|e| ShadowError::ConfigError(e.to_string()))?;

        Ok(())
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn aliases(&self) -> &Aliases {
        &self.aliases
    }

    pub fn add(&mut self, alias: Alias) -> Result<()> {
        let name = alias.name();
        if self.aliases.contains(name) {
            return Err(ShadowError::AliasExists(name.to_string()));
        }
        self.aliases.insert(name.to_string(), alias);
        self.save()?;
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> Result<()> {
        if !self.aliases.contains(name) {
            return Err(ShadowError::AliasNotFound(name.to_string()));
        }
        self.aliases.remove(name);
        self.save()?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .expect("Could not find config directory")
            .join("shdw/config.toml")
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Settings {
    #[serde(default = "Settings::default_bin_path")]
    bin_path: PathBuf,
    #[serde(default)]
    always_use_raw: bool,
}

impl Settings {
    pub fn new(bin_path: PathBuf, always_use_raw: bool) -> Self {
        Self {
            bin_path,
            always_use_raw,
        }
    }

    fn default() -> Self {
        Self::new(Self::default_bin_path(), false)
    }

    pub fn bin_path(&self) -> &PathBuf {
        &self.bin_path
    }

    fn default_bin_path() -> PathBuf {
        dirs::executable_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".local/bin")))
            .expect("Could not determine binary directory")
    }
}
