use std::path::PathBuf;

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Start the server
    Start {
        /// Host address to bind to
        #[arg(short, long)]
        pub host: Option<String>,

        /// Port to listen on
        #[arg(short, long)]
        pub port: Option<u16>,

        /// Run in daemon mode
        #[arg(short, long)]
        pub daemon: bool,

        /// Path to config file
        #[arg(short, long)]
        pub config_path: Option<PathBuf>,
    },

    /// Stop the server
    Stop,

    /// Check server status
    Status,

    /// Initialize configuration
    InitConfig {
        /// Path to create config file
        #[arg(short, long)]
        pub config_path: Option<PathBuf>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_variants() {
        let start_cmd = Command::Start {
            host: Some("localhost".to_string()),
            port: Some(8080),
            daemon: true,
            config_path: None,
        };
        
        let stop_cmd = Command::Stop;
        let status_cmd = Command::Status;
        
        let init_cmd = Command::InitConfig {
            config_path: Some(PathBuf::from("/tmp/config.toml")),
        };
        
        // Just testing that we can create all variants
        assert!(matches!(start_cmd, Command::Start { .. }));
        assert!(matches!(stop_cmd, Command::Stop));
        assert!(matches!(status_cmd, Command::Status));
        assert!(matches!(init_cmd, Command::InitConfig { .. }));
    }
}
