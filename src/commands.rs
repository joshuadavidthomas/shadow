use crate::aliases::Alias;
use crate::config::Config;
use crate::error::ExitCode;
use clap::Parser;
use std::path::PathBuf;

#[derive(Clone, Debug, Parser)]
pub struct Add {
    /// Name of the alias
    name: String,
    /// Command to execute
    command: String,
    /// Description of the alias
    #[arg(long)]
    description: Option<String>,
    /// Directory to create symlink in
    #[arg(long)]
    bin_path: Option<PathBuf>,
}

impl Add {
    pub fn execute(&self, mut config: Config) -> ExitCode {
        let bin_path = match &self.bin_path {
            Some(p) if p == config.settings().bin_path() => None,
            Some(p) => Some(p.clone()),
            None => None,
        };

        let alias = Alias::new(
            self.name.clone(),
            self.command.clone(),
            self.description.clone(),
            bin_path,
        );

        if let Err(e) = alias.create_symlink(config.settings()) {
            eprintln!("{}", e);
            return e.into();
        }

        match config.add(alias) {
            Ok(()) => {
                println!("Added alias: {}", self.name);
                ExitCode::Success
            }
            Err(e) => {
                eprintln!("{}", e);
                e.into()
            }
        }
    }
}

#[derive(Clone, Debug, Parser)]
pub struct Remove {
    /// Name of the alias to remove
    name: String,
    /// Directory containing the symlink
    #[arg(long)]
    bin_path: Option<PathBuf>,
}

impl Remove {
    pub fn execute(&self, mut config: Config) -> ExitCode {
        let alias = match config.aliases().get(&self.name) {
            Some(alias) => alias,
            None => {
                eprintln!("Alias not found: {}", self.name);
                return ExitCode::CommandNotFound;
            }
        };

        if let Err(e) = alias.remove_symlink(config.settings()) {
            eprintln!("{}", e);
            return e.into();
        }

        match config.remove(&self.name) {
            Ok(()) => {
                println!("Removed alias: {}", self.name);
                ExitCode::Success
            }
            Err(e) => {
                eprintln!("{}", e);
                e.into()
            }
        }
    }
}

#[derive(Clone, Debug, Parser)]
pub struct List;

impl List {
    pub fn execute(&self, config: Config) -> ExitCode {
        match config.aliases().is_empty() {
            true => println!("No aliases configured"),
            false => {
                let mut aliases: Vec<_> = config.aliases().values().collect();
                aliases.sort_by(|a, b| a.name().cmp(b.name()));
                for alias in aliases {
                    println!("{}", alias);
                }
            }
        }
        ExitCode::Success
    }
}
