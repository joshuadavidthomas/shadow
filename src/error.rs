use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    ConfigError = 2,
    InvalidArguments = 64,
    CommandNotFound = 127,
    CommandFailed = 128,
    DuplicateCommand = 129,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> i32 {
        code as i32
    }
}

impl From<ShadowError> for ExitCode {
    fn from(error: ShadowError) -> Self {
        match error {
            ShadowError::ShadowNotFound(_) => ExitCode::CommandNotFound,
            ShadowError::CommandExecutionError(_) => ExitCode::CommandFailed,
            ShadowError::ConfigError(_) => ExitCode::ConfigError,
            ShadowError::InvalidReplacement(_) => ExitCode::InvalidArguments,
            ShadowError::DuplicateCommand(_) => ExitCode::DuplicateCommand,
        }
    }
}

#[derive(Error, Debug)]
pub enum ShadowError {
    #[error("No shadow found for command: {0}")]
    ShadowNotFound(String),
    #[error("Command already shadowed: {0}")]
    DuplicateCommand(String),
    #[error("Failed to execute command: {0}")]
    CommandExecutionError(#[from] std::io::Error),
    #[error("Failed to load config: {0}")]
    ConfigError(String),
    #[error("Invalid replacement command: {0}")]
    InvalidReplacement(String),
}

pub type Result<T> = std::result::Result<T, ShadowError>;
