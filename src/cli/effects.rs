use std::env;
use crate::cli::{Args};
use crate::config::Config;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Failed to parse arguments: {0}")]
    ParseError(#[from] ParseError),
    
    #[error("Failed to execute command: {0}")]
    ExecutionError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
}

pub fn get_args_from_env() -> Result<Args, CliError> {
    Ok(Args::parse())
}

pub fn load_config(path: &PathBuf) -> Result<Config, CliError> {
    Config::load(path).map_err(CliError::from)
}

pub fn create_pid_file(path: &PathBuf) -> Result<(), CliError> {
    use std::fs::File;
    use std::io::Write;
    
    let pid = std::process::id();
    File::create(path)
        .and_then(|mut f| writeln!(f, "{}", pid))
        .map_err(|e| CliError::ExecutionError(format!("Failed to create PID file: {}", e)))
}
