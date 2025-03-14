mod effects;
mod pure;

use clap::Parser;
use std::path::PathBuf;

pub use effects::CliError;
pub use pure::Command;

pub struct Cli;

impl Cli {
    pub fn new() -> Self {
        Cli
    }

    pub fn execute(&self, command: Command) -> Result<String, CliError> {
        // Implement the command execution logic here
        Ok(format!("Executed command: {:?}", command))
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
