mod cli;
mod commands;
mod config;
mod error;
mod shadows;

use crate::cli::Cli;
use crate::config::Config;
use crate::error::ExitCode;
use std::env;
use std::path::Path;
use std::process::exit;

fn main() {
    // For debugging
    println!("Args: {:?}", std::env::args().collect::<Vec<_>>());
    println!("Program: {:?}", std::env::current_exe().unwrap());

    let args = env::args().next().unwrap();
    let program_name = Path::new(&args)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("shadow");
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            exit(ExitCode::ConfigError.into());
        }
    };

    let exit_code = match program_name {
        "shadow" => Cli::execute(config),
        command => Cli::execute_shadowed(config, command),
    };

    exit(exit_code.into())
}