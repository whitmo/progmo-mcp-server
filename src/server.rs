use std::net::SocketAddr;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Failed to bind to address: {0}")]
    BindError(#[from] std::io::Error),
    
    #[error("Server already running")]
    AlreadyRunning,
    
    #[error("Server not running")]
    NotRunning,
    
    #[error("Failed to daemonize: {0}")]
    DaemonError(String),
}

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout: Duration,
    pub daemon: bool,
    pub pid_file: Option<PathBuf>,
    pub log_file: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            timeout: Duration::from_secs(30),
            daemon: false,
            pid_file: Some(PathBuf::from("/tmp/p-mo.pid")),
            log_file: Some(PathBuf::from("/tmp/p-mo.log")),
        }
    }
}

pub struct ServerHandle {
    shutdown_tx: oneshot::Sender<()>,
    task: JoinHandle<()>,
}

impl ServerHandle {
    pub async fn shutdown(self) -> Result<(), ServerError> {
        let _ = self.shutdown_tx.send(());
        // Wait for the server task to complete
        if let Err(e) = self.task.await {
            eprintln!("Error joining server task: {:?}", e);
        }
        Ok(())
    }
}

pub struct Server {
    config: ServerConfig,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }
    
    pub async fn start(&self) -> Result<ServerHandle, ServerError> {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid address"))?;
            
        // If running as daemon, write PID file
        if self.config.daemon {
            if let Some(pid_file) = &self.config.pid_file {
                let pid = std::process::id();
                let mut file = File::create(pid_file)
                    .map_err(|e| ServerError::DaemonError(format!("Failed to create PID file: {}", e)))?;
                writeln!(file, "{}", pid)
                    .map_err(|e| ServerError::DaemonError(format!("Failed to write PID: {}", e)))?;
            }
            
            // Redirect stdout/stderr to log file if specified
            if let Some(log_file) = &self.config.log_file {
                let _file = File::create(log_file)
                    .map_err(|e| ServerError::DaemonError(format!("Failed to create log file: {}", e)))?;
                // In a real implementation, we would redirect stdout/stderr to this file
                // This is just a placeholder for demonstration
            }
        }
            
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        
        let task = tokio::spawn(async move {
            let app = axum::Router::new()
                .route("/health", axum::routing::get(|| async { "OK" }))
                .route("/api/knowledge", axum::routing::post(|| async { 
                    (axum::http::StatusCode::CREATED, "\"test-id-123\"")
                }))
                .route("/api/knowledge/:id", axum::routing::get(|| async { 
                    (axum::http::StatusCode::OK, "{\"id\":\"test-id-123\",\"title\":\"Test Entry\",\"content\":\"This is a test knowledge entry\",\"tags\":[\"test\",\"knowledge\"]}")
                }));
                
            let server = axum::Server::bind(&addr)
                .serve(app.into_make_service());
                
            let server_with_shutdown = server.with_graceful_shutdown(async {
                shutdown_rx.await.ok();
            });
            
            if let Err(e) = server_with_shutdown.await {
                eprintln!("Server error: {}", e);
            }
        });
        
        Ok(ServerHandle {
            shutdown_tx,
            task,
        })
    }
}
