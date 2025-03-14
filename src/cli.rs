use thiserror::Error;
use std::path::PathBuf;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Failed to execute command: {0}")]
    ExecutionError(String),
    
    #[error("Invalid command")]
    InvalidCommand,
    
    #[error("Configuration error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
}

pub enum Command {
    Start {
        host: Option<String>,
        port: Option<u16>,
        daemon: bool,
        config_path: Option<PathBuf>,
    },
    Stop,
    Status,
    InitConfig {
        config_path: Option<PathBuf>,
    },
}

pub struct Cli {
    // Configuration and state
    is_running: std::sync::atomic::AtomicBool,
}

impl Cli {
    pub fn new() -> Self {
        Self {
            is_running: std::sync::atomic::AtomicBool::new(false),
        }
    }
    
    pub fn execute(&self, command: Command) -> Result<String, CliError> {
        match command {
            Command::Start { host, port, daemon, config_path } => {
                let (final_host, final_port, final_daemon) = if let Some(path) = config_path {
                    // Load config if path is provided
                    if let Ok(config) = Config::load(&path) {
                        (
                            host.unwrap_or(config.server.host),
                            port.unwrap_or(config.server.port),
                            daemon || config.server.daemon
                        )
                    } else {
                        (
                            host.unwrap_or_else(|| "127.0.0.1".to_string()),
                            port.unwrap_or(8080),
                            daemon
                        )
                    }
                } else {
                    (
                        host.unwrap_or_else(|| "127.0.0.1".to_string()),
                        port.unwrap_or(8080),
                        daemon
                    )
                };
                
                // Set server as running
                self.is_running.store(true, std::sync::atomic::Ordering::SeqCst);
                
                let daemon_msg = if final_daemon {
                    " in daemon mode"
                } else {
                    ""
                };
                
                let config_msg = if let Some(path) = config_path {
                    format!(" (using config from {})", path.display())
                } else {
                    "".to_string()
                };
                
                // Simulate starting server
                Ok(format!("Server started on {}:{}{}{}", 
                    final_host, 
                    final_port, 
                    daemon_msg,
                    config_msg
                ))
            },
            Command::Stop => {
                // Set server as stopped
                self.is_running.store(false, std::sync::atomic::Ordering::SeqCst);
                
                // Simulate stopping server
                Ok("Server stopped".to_string())
            },
            Command::Status => {
                // Check if server is running
                if self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
                    Ok("Server is running".to_string())
                } else {
                    Ok("Server is stopped".to_string())
                }
            },
            Command::InitConfig { config_path } => {
                let path = if let Some(path) = config_path {
                    // Ensure parent directory exists
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| CliError::ExecutionError(format!("Failed to create directory: {}", e)))?;
                    }
                    path
                } else {
                    crate::config::Config::create_default_config()?
                };
                
                // Create default config if it doesn't exist
                if !path.exists() {
                    let config = crate::config::Config::default();
                    config.save(&path)?;
                    Ok(format!("Created default configuration at {}", path.display()))
                } else {
                    Ok(format!("Configuration already exists at {}", path.display()))
                }
            }
        }
    }
}
