use crate::config::Settings;
use crate::error::{ExitCode, Result, ShadowError};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Aliases(Vec<Alias>);

impl Aliases {
    pub fn find<S: AsRef<str>>(&self, original: S) -> Result<&Alias> {
        self.0
            .iter()
            .find(|s| s.original == original.as_ref())
            .ok_or_else(|| ShadowError::AliasNotFound(original.as_ref().to_string()))
    }

    pub fn contains<S: AsRef<str>>(&self, original: S) -> bool {
        self.find(original).is_ok()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for Aliases {
    type Target = Vec<Alias>;

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
    type Item = Alias;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Aliases {
    type Item = &'a Alias;
    type IntoIter = std::slice::Iter<'a, Alias>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Alias {
    original: String,
    replacement: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    bin_path: Option<PathBuf>,
}

impl Alias {
    pub fn new(original: String, replacement: String, bin_path: Option<PathBuf>) -> Self {
        Self {
            original,
            replacement,
            bin_path,
        }
    }

    pub fn original(&self) -> &String {
        &self.original
    }

    pub fn replacement(&self) -> &String {
        &self.replacement
    }

    pub fn bin_path(&self) -> &Option<PathBuf> {
        &self.bin_path
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
            fs::remove_file(&link_path)?;
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

    fn link_path(&self, bin_path: &Path) -> PathBuf {
        let link_name = if cfg!(windows) {
            format!("{}.exe", self.original)
        } else {
            self.original.clone()
        };
        bin_path.join(link_name)
    }

    pub fn execute(&self, args: &[String], raw: bool) -> ExitCode {
        if raw {
            self.execute_original(args)
        } else {
            self.execute_alias(args)
        }
    }

    fn execute_original(&self, args: &[String]) -> ExitCode {
        match Command::new(&self.original).args(args).status() {
            Ok(status) => match status.code() {
                Some(0) => ExitCode::Success,
                Some(_) => ExitCode::CommandFailed,
                None => ExitCode::CommandFailed,
            },
            Err(e) => {
                eprintln!("Failed to execute {}: {}", self.original, e);
                ExitCode::CommandFailed
            }
        }
    }

    fn execute_alias(&self, args: &[String]) -> ExitCode {
        let parts: Vec<&str> = self.replacement.split_whitespace().collect();
        let (cmd, base_args) = match parts.split_first() {
            Some(parts) => parts,
            None => {
                eprintln!("Invalid replacement command: {}", self.replacement);
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
}

impl fmt::Display for Alias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} → {}", self.original, self.replacement)?;
        if let Some(path) = &self.bin_path {
            write!(f, " (in {})", path.display())?;
        }
        Ok(())
    }
}
