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
            let mut host = None;
            let mut port = None;
            let mut daemon = false;
            
            // Parse remaining arguments
            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--host" | "-h" if i + 1 < args.len() => {
                        host = Some(args[i + 1].clone());
                        i += 2;
                    },
                    "--port" | "-p" if i + 1 < args.len() => {
                        port = args[i + 1].parse::<u16>().ok();
                        i += 2;
                    },
                    "--daemon" | "-d" => {
                        daemon = true;
                        i += 1;
                    },
                    _ => {
                        i += 1;
                    }
                }
            }
            
            cli.execute(Command::Start { host, port, daemon })
        },
        Some("stop") => cli.execute(Command::Stop),
        Some("status") => cli.execute(Command::Status),
        _ => {
            println!("Usage: p-mo [command] [options]");
            println!("Commands:");
            println!("  start     Start the p-mo server");
            println!("    --host, -h <host>    Specify host (default: 127.0.0.1)");
            println!("    --port, -p <port>    Specify port (default: 8080)");
            println!("    --daemon, -d         Run as daemon in background");
            println!("  stop      Stop the p-mo server");
            println!("  status    Check p-mo server status");
            Ok("".to_string())
        }
    };
    
    match result {
        Ok(message) if !message.is_empty() => println!("{}", message),
        Err(err) => eprintln!("Error: {}", err),
        _ => {}
    }
}
