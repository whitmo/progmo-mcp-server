use crate::cli::{Cli, Command, CliError};
use crate::config::Config;
use std::path::PathBuf;

pub struct App {
    cli: Cli,
    config: Option<Config>,
}

impl App {
    pub fn new() -> Self {
        Self {
            cli: Cli::new(),
            config: None,
        }
    }

    pub fn load_config(&mut self, config_path: &Option<PathBuf>) -> Result<(), CliError> {
        let config_path = config_path.clone().unwrap_or_else(Config::default_path);
        self.config = Some(Config::load(&config_path).map_err(CliError::from)?);
        Ok(())
    }

    pub fn execute(&mut self, command: Command) -> Result<String, CliError> {
        match command {
            Command::Start { host, port, daemon, config_path } => {
                self.load_config(&config_path)?;
                self.cli.execute(Command::Start { host, port, daemon, config_path })
            },
            other => self.cli.execute(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_app_initialization() {
        let app = App::new();
        assert!(app.config.is_none());
    }

    #[test]
    fn test_app_config_loading() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        // Create a test config file
        let config_content = r#"
[server]
host = "127.0.0.1"
port = 8080
"#;
        fs::write(&config_path, config_content).unwrap();

        let mut app = App::new();
        assert!(app.load_config(Some(config_path)).is_ok());
        assert!(app.config.is_some());
    }

    #[test]
    fn test_app_execute_stop_command() {
        let mut app = App::new();
        let result = app.execute(Command::Stop);
        assert!(result.is_ok());
    }

    #[test]
    fn test_app_execute_status_command() {
        let mut app = App::new();
        let result = app.execute(Command::Status);
        assert!(result.is_ok());
    }

    #[test]
    fn test_app_execute_init_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("new_config.toml");

        let mut app = App::new();
        let result = app.execute(Command::InitConfig { 
            config_path: Some(config_path.clone()) 
        });
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_app_config_loading_with_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent_config.toml");

        let mut app = App::new();
        let result = app.load_config(Some(config_path));
        
        assert!(result.is_err());
    }

    #[test]
    fn test_app_execute_start_with_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("start_config.toml");
        
        let config_content = r#"
[server]
host = "127.0.0.1"
port = 8080
"#;
        fs::write(&config_path, config_content).unwrap();

        let mut app = App::new();
        let result = app.execute(Command::Start {
            host: None,
            port: None,
            daemon: false,
            config_path: Some(config_path),
        });
        
        assert!(result.is_ok());
    }
}
