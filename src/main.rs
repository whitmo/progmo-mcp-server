use p_mo::cli::{Cli, Command};
use std::env;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Create CLI instance
    let cli = Cli::new();
    
    // Simple command parsing for now
    let result = match args.get(1).map(|s| s.as_str()) {
        Some("start") => {
            let host = args.get(2).map(|s| s.to_string());
            let port = args.get(3).and_then(|s| s.parse::<u16>().ok());
            cli.execute(Command::Start { host, port })
        },
        Some("stop") => cli.execute(Command::Stop),
        Some("status") => cli.execute(Command::Status),
        _ => {
            println!("Usage: p-mo [start|stop|status]");
            Ok("".to_string())
        }
    };
    
    match result {
        Ok(message) if !message.is_empty() => println!("{}", message),
        Err(err) => eprintln!("Error: {}", err),
        _ => {}
    }
}
