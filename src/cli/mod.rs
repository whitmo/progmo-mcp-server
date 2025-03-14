use clap::Parser;
use std::path::PathBuf;

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

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Start the server
    Start {
        /// Host address to bind to
        #[arg(short, long)]
        host: Option<String>,

        /// Port to listen on
        #[arg(short, long)]
        port: Option<u16>,

        /// Run in daemon mode
        #[arg(short, long)]
        daemon: bool,

        /// Path to config file
        #[arg(short, long)]
        config_path: Option<PathBuf>,
    },

    /// Stop the server
    Stop,

    /// Check server status
    Status,

    /// Initialize configuration
    InitConfig {
        /// Path to create config file
        #[arg(short, long)]
        config_path: Option<PathBuf>,
    },
}
