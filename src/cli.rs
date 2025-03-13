use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Failed to execute command: {0}")]
    ExecutionError(String),
    
    #[error("Invalid command")]
    InvalidCommand,
}

pub enum Command {
    Start {
        host: Option<String>,
        port: Option<u16>,
        daemon: bool,
    },
    Stop,
    Status,
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
            Command::Start { host, port, daemon } => {
                // In a real implementation, this would start the server
                // For testing, we'll simulate it
                let host = host.unwrap_or_else(|| "127.0.0.1".to_string());
                let port = port.unwrap_or(8080);
                
                // Set server as running
                self.is_running.store(true, std::sync::atomic::Ordering::SeqCst);
                
                let daemon_msg = if daemon {
                    " in daemon mode"
                } else {
                    ""
                };
                
                // Simulate starting server
                Ok(format!("Server started on {}:{}{}", host, port, daemon_msg))
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
            }
        }
    }
}
