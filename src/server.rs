use std::net::SocketAddr;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Failed to bind to address: {0}")]
    BindError(#[from] std::io::Error),
    
    #[error("Server already running")]
    AlreadyRunning,
    
    #[error("Server not running")]
    NotRunning,
}

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout: Duration,
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
