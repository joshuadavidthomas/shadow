use crate::commands::Commands;
use crate::config::Config;
use crate::error::ExitCode;
use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug)]
pub struct ShadowedArgs {
    args: Vec<String>,
    is_raw: bool,
}

impl ShadowedArgs {
    pub fn from_env() -> Self {
        let args: Vec<String> = env::args().skip(1).collect();
        let is_raw = args.contains(&"--raw".to_string()) || args.contains(&"-R".to_string());
        let args = args
            .into_iter()
            .filter(|arg| arg != "--raw" && arg != "-R")
            .collect();

        Self { args, is_raw }
    }
}

impl Cli {
    pub fn execute(config: Config) -> ExitCode {
        let cli = Self::parse();
        match &cli.command {
            Some(cmd) => cmd.execute(config),
            None => {
                println!("Use --help for usage information");
                ExitCode::InvalidArguments
            }
        }
    }

    pub fn execute_shadowed(config: Config, command: &str) -> ExitCode {
        let args = ShadowedArgs::from_env();
        match config
            .shadows()
            .find(command)
            .map(|shadow| shadow.execute(&args.args, args.is_raw))
        {
            Ok(code) => code,
            Err(e) => {
                eprintln!("{}", e);
                e.into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    impl ShadowedArgs {
        pub fn new(args: Vec<String>) -> Self {
            let is_raw = args.contains(&"--raw".to_string()) || args.contains(&"-R".to_string());
            let args = args
                .into_iter()
                .filter(|arg| arg != "--raw" && arg != "-R")
                .collect();

            Self { args, is_raw }
        }
    }

    impl Cli {
        pub fn execute_shadowed_with_args(
            config: Config,
            _temp_dir: TempDir, // Add this to keep it alive
            command: &str,
            args: ShadowedArgs,
        ) -> ExitCode {
            match config
                .shadows()
                .find(command)
                .map(|shadow| shadow.execute(&args.args, args.is_raw))
            {
                Ok(code) => code,
                Err(e) => {
                    eprintln!("{}", e);
                    e.into()
                }
            }
        }

        pub fn execute_with_args(config: Config, _temp_dir: TempDir, args: Vec<&str>) -> ExitCode {
            let cli = Self::try_parse_from(args).unwrap();
            match &cli.command {
                Some(cmd) => cmd.execute(config),
                None => {
                    println!("Use --help for usage information");
                    ExitCode::InvalidArguments
                }
            }
        }
    }

    mod shadowed_args {
        use super::*;

        #[test]
        fn test_normal_args() {
            let args = ShadowedArgs::new(vec!["arg1".to_string(), "arg2".to_string()]);
            assert_eq!(args.args, vec!["arg1", "arg2"]);
            assert!(!args.is_raw);
        }

        #[test]
        fn test_raw_long_flag() {
            let args = ShadowedArgs::new(vec!["--raw".to_string(), "arg1".to_string()]);
            assert_eq!(args.args, vec!["arg1"]);
            assert!(args.is_raw);
        }

        #[test]
        fn test_raw_short_flag() {
            let args = ShadowedArgs::new(vec!["-R".to_string(), "arg1".to_string()]);
            assert_eq!(args.args, vec!["arg1"]);
            assert!(args.is_raw);
        }

        #[test]
        fn test_raw_flag_middle() {
            let args = ShadowedArgs::new(vec![
                "arg1".to_string(),
                "--raw".to_string(),
                "arg2".to_string(),
            ]);
            assert_eq!(args.args, vec!["arg1", "arg2"]);
            assert!(args.is_raw);
        }

        #[test]
        fn test_multiple_raw_flags() {
            let args = ShadowedArgs::new(vec![
                "--raw".to_string(),
                "-R".to_string(),
                "arg1".to_string(),
            ]);
            assert_eq!(args.args, vec!["arg1"]);
            assert!(args.is_raw);
        }

        #[test]
        fn test_empty_args() {
            let args = ShadowedArgs::new(vec![]);
            assert!(args.args.is_empty());
            assert!(!args.is_raw);
        }

        #[test]
        fn test_only_raw_flag() {
            let args = ShadowedArgs::new(vec!["--raw".to_string()]);
            assert!(args.args.is_empty());
            assert!(args.is_raw);
        }
    }
}
