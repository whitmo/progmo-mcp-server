use crate::cli::{Args};
use crate::config::Config;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    
    #[error("Failed to execute command: {0}")]
    ExecutionError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
}

#[allow(dead_code)]
pub fn get_args_from_env() -> Result<Args, CliError> {
    Ok(Args::parse())
}

#[allow(dead_code)]
pub fn load_config(path: &PathBuf) -> Result<Config, CliError> {
    Config::load(path).map_err(CliError::from)
}

#[allow(dead_code)]
pub fn create_pid_file(path: &PathBuf) -> Result<(), CliError> {
    use std::fs::File;
    use std::io::Write;
    
    let pid = std::process::id();
    File::create(path)
        .and_then(|mut f| writeln!(f, "{}", pid))
        .map_err(|e| CliError::ExecutionError(format!("Failed to create PID file: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    #[ignore] // Ignore this test as it tries to parse CLI args which can cause issues in test suite
    fn test_get_args_from_env() {
        // This is a simple wrapper around Args::parse(), so we just verify it doesn't panic
        // In a real test, we would mock the CLI args
        let _ = get_args_from_env();
    }
    
    #[test]
    fn test_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        // Create a test config file
        let config_content = r#"
[server]
host = "127.0.0.1"
port = 8080
"#;
        std::fs::write(&config_path, config_content).unwrap();
        
        let result = load_config(&config_path);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_create_pid_file() {
        let temp_dir = TempDir::new().unwrap();
        let pid_path = temp_dir.path().join("test.pid");
        
        let result = create_pid_file(&pid_path);
        assert!(result.is_ok());
        assert!(pid_path.exists());
        
        // Verify the PID file contains a number
        let content = std::fs::read_to_string(&pid_path).unwrap();
        let pid: u32 = content.trim().parse().unwrap();
        assert!(pid > 0);
    }
}
