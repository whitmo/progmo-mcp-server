use p_mo::cli::{Cli, Command};
use p_mo::config::Config;
use std::env;
use std::path::PathBuf;

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
            let mut config_path = None;
            
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
                    "--config" | "-c" if i + 1 < args.len() => {
                        config_path = Some(PathBuf::from(&args[i + 1]));
                        i += 2;
                    },
                    _ => {
                        i += 1;
                    }
                }
            }
            
            // If config path is provided, load configuration
            if let Some(path) = &config_path {
                if path.exists() {
                    match Config::load(path) {
                        Ok(config) => {
                            // Command line arguments take precedence over config file
                            if host.is_none() {
                                host = Some(config.server.host);
                            }
                            if port.is_none() {
                                port = Some(config.server.port);
                            }
                            if !daemon {
                                daemon = config.server.daemon;
                            }
                        },
                        Err(e) => {
                            eprintln!("Error loading configuration: {}", e);
                        }
                    }
                } else {
                    eprintln!("Config file not found: {}", path.display());
                }
            } else {
                // Try to load default config if no config path is provided
                let default_path = Config::default_path();
                if default_path.exists() {
                    match Config::load(&default_path) {
                        Ok(config) => {
                            // Command line arguments take precedence over config file
                            if host.is_none() {
                                host = Some(config.server.host);
                            }
                            if port.is_none() {
                                port = Some(config.server.port);
                            }
                            if !daemon {
                                daemon = config.server.daemon;
                            }
                        },
                        Err(e) => {
                            eprintln!("Error loading default configuration: {}", e);
                        }
                    }
                }
            }
            
            cli.execute(Command::Start { host, port, daemon, config_path })
        },
        Some("stop") => cli.execute(Command::Stop),
        Some("status") => cli.execute(Command::Status),
        Some("init-config") => {
            let mut config_path = None;
            
            // Parse remaining arguments
            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--path" | "-p" if i + 1 < args.len() => {
                        config_path = Some(PathBuf::from(&args[i + 1]));
                        i += 2;
                    },
                    _ => {
                        i += 1;
                    }
                }
            }
            
            cli.execute(Command::InitConfig { config_path })
        },
        _ => {
            println!("Usage: p-mo [command] [options]");
            println!("Commands:");
            println!("  start         Start the p-mo server");
            println!("    --host, -h <host>      Specify host (default: 127.0.0.1)");
            println!("    --port, -p <port>      Specify port (default: 8080)");
            println!("    --daemon, -d           Run as daemon in background");
            println!("    --config, -c <path>    Specify config file path");
            println!("  stop          Stop the p-mo server");
            println!("  status        Check p-mo server status");
            println!("  init-config   Create a default configuration file");
            println!("    --path, -p <path>      Specify config file path");
            Ok("".to_string())
        }
    };
    
    match result {
        Ok(message) if !message.is_empty() => println!("{}", message),
        Err(err) => eprintln!("Error: {}", err),
        _ => {}
    }
}
