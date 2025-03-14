use clap::Parser;
use p_mo::app::App;
use p_mo::cli::{Command, CliError};
use tracing_subscriber;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn run() -> Result<(), CliError> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    let mut app = App::new();
    
    let result = app.execute(args.command)?;
    if !result.is_empty() {
        println!("{}", result);
    }
    
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_command_parsing() {
        let args = Args::parse_from(&["prog", "start"]);
        match args.command {
            Command::Start { host: None, port: None, daemon: false, config_path: None } => (),
            _ => panic!("Unexpected command parsing result"),
        }
    }

    #[test]
    fn test_command_parsing_with_options() {
        let args = Args::parse_from(&[
            "prog", "start",
            "--host", "localhost",
            "--port", "8080",
            "--daemon",
        ]);
        
        match args.command {
            Command::Start { 
                host: Some(h),
                port: Some(p),
                daemon: true,
                config_path: None,
            } => {
                assert_eq!(h, "localhost");
                assert_eq!(p, 8080);
            },
            _ => panic!("Unexpected command parsing result"),
        }
    }
}
