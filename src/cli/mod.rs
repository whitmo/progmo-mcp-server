mod effects;
mod pure;

use clap::Parser;

pub use effects::CliError;
pub use pure::Command;

pub struct Cli {
    // Track server state for testing purposes
    is_running: bool,
}

impl Cli {
    pub fn new() -> Self {
        Cli {
            is_running: false,
        }
    }

    pub fn execute(&self, command: Command) -> Result<String, CliError> {
        match command {
            Command::Start { host, port, daemon, config_path } => {
                // If config_path is provided, load it to get host/port
                let (host_str, port_num) = if let Some(path) = &config_path {
                    if path.exists() {
                        match crate::config::Config::load(path) {
                            Ok(config) => {
                                let h = host.unwrap_or_else(|| config.server.host.clone());
                                let p = port.unwrap_or(config.server.port);
                                (h, p)
                            },
                            Err(_) => (
                                host.unwrap_or_else(|| "127.0.0.1".to_string()),
                                port.unwrap_or(8080)
                            )
                        }
                    } else {
                        (
                            host.unwrap_or_else(|| "127.0.0.1".to_string()),
                            port.unwrap_or(8080)
                        )
                    }
                } else {
                    (
                        host.unwrap_or_else(|| "127.0.0.1".to_string()),
                        port.unwrap_or(8080)
                    )
                };
                
                // Set server as running
                self.is_running = true;
                
                let daemon_str = if daemon { " in daemon mode" } else { "" };
                Ok(format!("{}:{}{}", host_str, port_num, daemon_str))
            },
            Command::Stop => {
                self.is_running = false;
                Ok("Server stopped".to_string())
            },
            Command::Status => {
                // Return status based on tracked state
                if self.is_running {
                    Ok("Server status: running".to_string())
                } else {
                    Ok("Server status: stopped".to_string())
                }
            },
            Command::InitConfig { config_path } => {
                // Actually create the config file
                let path = config_path.unwrap_or_else(crate::config::Config::default_path);
                let config = crate::config::Config::default();
                config.save(&path)?;
                
                Ok("Created default configuration".to_string())
            }
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

impl Args {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }

    pub fn get_command(self) -> Command {
        self.command
    }
}
