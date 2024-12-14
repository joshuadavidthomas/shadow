use crate::config::Settings;
use crate::error::{ExitCode, Result, ShadowError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug, Default, Serialize)]
pub struct Aliases(HashMap<String, Alias>);

impl Aliases {
    pub fn find<S: AsRef<str>>(&self, original: S) -> Result<&Alias> {
        self.0
            .get(original.as_ref())
            .ok_or_else(|| ShadowError::AliasNotFound(original.as_ref().to_string()))
    }

    pub fn contains<S: AsRef<str>>(&self, original: S) -> bool {
        self.0.contains_key(original.as_ref())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn values(&self) -> std::collections::hash_map::Values<'_, String, Alias> {
        self.0.values()
    }

    pub fn values_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, String, Alias> {
        self.0.values_mut()
    }
}

impl Deref for Aliases {
    type Target = HashMap<String, Alias>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Aliases {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Aliases {
    type Item = (String, Alias);
    type IntoIter = std::collections::hash_map::IntoIter<String, Alias>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Aliases {
    type Item = (&'a String, &'a Alias);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Alias>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'de> Deserialize<'de> for Aliases {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut map = HashMap::<String, Alias>::deserialize(deserializer)?;

        for (key, alias) in map.iter_mut() {
            alias.name = key.clone();
        }

        Ok(Aliases(map))
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Alias {
    #[serde(skip)]
    name: String,
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bin_path: Option<PathBuf>,
}

#[derive(Deserialize)]
struct AliasDef {
    command: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    bin_path: Option<PathBuf>,
}

impl Alias {
    pub fn new(
        name: String,
        command: String,
        description: Option<String>,
        bin_path: Option<PathBuf>,
    ) -> Self {
        Self {
            name,
            command,
            description,
            bin_path,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn bin_path(&self) -> &Option<PathBuf> {
        &self.bin_path
    }

    fn link_path(&self, bin_path: &Path) -> PathBuf {
        let link_name = if cfg!(windows) {
            format!("{}.exe", self.name)
        } else {
            self.name.clone()
        };
        bin_path.join(link_name)
    }

    pub fn execute(&self, args: &[String], raw: bool) -> ExitCode {
        if raw {
            self.execute_original(args)
        } else {
            self.execute_command(args)
        }
    }

    fn execute_original(&self, args: &[String]) -> ExitCode {
        match Command::new(&self.name).args(args).status() {
            Ok(status) => match status.code() {
                Some(0) => ExitCode::Success,
                Some(_) => ExitCode::CommandFailed,
                None => ExitCode::CommandFailed,
            },
            Err(e) => {
                eprintln!("Failed to execute {}: {}", self.name, e);
                ExitCode::CommandFailed
            }
        }
    }

    fn execute_command(&self, args: &[String]) -> ExitCode {
        let parts: Vec<&str> = self.command.split_whitespace().collect();
        let (cmd, base_args) = match parts.split_first() {
            Some(parts) => parts,
            None => {
                eprintln!("Invalid command: {}", self.command);
                return ExitCode::InvalidArguments;
            }
        };

        let all_args: Vec<String> = base_args
            .iter()
            .map(|&s| s.to_string())
            .chain(args.iter().cloned())
            .collect();

        match Command::new(cmd).args(all_args).status() {
            Ok(status) => match status.code() {
                Some(0) => ExitCode::Success,
                Some(_) => ExitCode::CommandFailed,
                None => ExitCode::CommandFailed,
            },
            Err(e) => {
                eprintln!("Failed to execute {}: {}", cmd, e);
                ExitCode::CommandFailed
            }
        }
    }

    pub fn create_symlink(&self, settings: &Settings) -> Result<()> {
        let bin_path = self
            .bin_path
            .as_deref()
            .unwrap_or_else(|| settings.bin_path());

        fs::create_dir_all(bin_path).map_err(|e| {
            ShadowError::ConfigError(format!("Failed to create bin directory: {}", e))
        })?;

        let target = env::current_exe().map_err(|e| {
            ShadowError::ConfigError(format!("Failed to get executable path: {}", e))
        })?;

        let link_path = self.link_path(bin_path);

        if link_path.exists() {
            if let Ok(existing_target) = fs::read_link(&link_path) {
                if existing_target == target {
                    return Ok(());
                }
            }
            // Either not a symlink or points somewhere else, remove it
            fs::remove_file(&link_path).map_err(|e| {
                ShadowError::ConfigError(format!("Failed to remove existing symlink: {}", e))
            })?;
        }

        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &link_path)?;

        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&target, &link_path)?;

        Ok(())
    }

    pub fn remove_symlink(&self, settings: &Settings) -> Result<()> {
        let bin_path = self
            .bin_path
            .as_deref()
            .unwrap_or_else(|| settings.bin_path());
        let link_path = self.link_path(bin_path);

        if link_path.exists() {
            fs::remove_file(&link_path)?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Alias {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let def = AliasDef::deserialize(deserializer)?;
        Ok(Alias {
            name: String::new(),
            command: def.command,
            description: def.description,
            bin_path: def.bin_path,
        })
    }
}

impl fmt::Display for Alias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} → {}", self.name, self.command)?;

        if let Some(desc) = &self.description {
            write!(f, " ({})", desc)?;
        }

        if let Some(path) = &self.bin_path {
            write!(f, " [in {}]", path.display())?;
        }

        Ok(())
    }
}
