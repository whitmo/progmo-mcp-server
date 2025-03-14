mod effects;
mod pure;

use clap::Parser;

pub use effects::CliError;
pub use pure::Command;

pub struct Cli;

impl Cli {
    pub fn new() -> Self {
        Cli
    }

    pub fn execute(&self, command: Command) -> Result<String, CliError> {
        match command {
            Command::Start { host, port, daemon, .. } => {
                let host_str = host.unwrap_or_else(|| "127.0.0.1".to_string());
                let port_num = port.unwrap_or(8080);
                let daemon_str = if daemon { " in daemon mode" } else { "" };
                Ok(format!("{}:{}{}", host_str, port_num, daemon_str))
            },
            Command::Stop => {
                Ok("Server stopped".to_string())
            },
            Command::Status => {
                Ok("Server status: running".to_string())
            },
            Command::InitConfig { .. } => {
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
