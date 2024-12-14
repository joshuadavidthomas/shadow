use crate::config::Config;
use crate::error::ExitCode;
use crate::shadows::Shadow;
use clap::Parser;
use clap::Subcommand;
use std::path::PathBuf;
use std::process::Command;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a new shadow
    Add(Add),
    /// Remove a shadow
    Remove(Remove),
    /// List all shadows
    List(List),
}

impl Commands {
    pub fn execute(&self, config: Config) -> ExitCode {
        match self {
            Commands::Add(cmd) => cmd.execute(config),
            Commands::Remove(cmd) => cmd.execute(config),
            Commands::List(cmd) => cmd.execute(config),
        }
    }
}

#[derive(Clone, Debug, Parser)]
pub struct Add {
    /// Original command to shadow
    original: String,
    /// Replacement command
    replacement: String,
    /// Directory to create symlink in
    #[arg(long)]
    bin_path: Option<PathBuf>,
}

impl Add {
    pub fn execute(&self, mut config: Config) -> ExitCode {
        if config.shadows().contains(&self.original) {
            eprintln!("Command already shadowed: {}", self.original);
            return ExitCode::DuplicateCommand;
        }

        let command = &self
            .original
            .split_whitespace()
            .next()
            .expect("command should be provided");

        if Command::new(command).output().is_err() {
            eprintln!("Command not found: {}", self.original);
            return ExitCode::CommandFailed;
        }

        let bin_path = match &self.bin_path {
            Some(p) if p == config.settings().bin_path() => None,
            Some(p) => Some(p.clone()),
            None => None,
        };

        let shadow = Shadow::new(self.original.clone(), self.replacement.clone(), bin_path);

        if let Err(e) = shadow.create_symlink(config.settings()) {
            eprintln!("{}", e);
            return e.into();
        }

        match config.add(shadow) {
            Ok(()) => {
                println!("Added shadow: {}", self.original);
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
    /// Command to un-shadow
    original: String,
    /// Directory containing the symlink
    #[arg(long)]
    bin_path: Option<PathBuf>,
}

impl Remove {
    pub fn execute(&self, mut config: Config) -> ExitCode {
        let shadow = match config.shadows().find(&self.original) {
            Ok(shadow) => shadow,
            Err(e) => {
                eprintln!("{}", e);
                return e.into();
            }
        };

        if let Err(e) = shadow.remove_symlink(config.settings()) {
            eprintln!("{}", e);
            return e.into();
        }

        match config.remove(&self.original) {
            Ok(()) => {
                println!("Removed shadow: {}", self.original);
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
        match config.shadows().is_empty() {
            true => println!("No shadows configured"),
            false => config
                .shadows()
                .iter()
                .for_each(|shadow| println!("{}", shadow)),
        }
        ExitCode::Success
    }
}
