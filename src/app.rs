use crate::metrics::server::MetricsService;
use std::net::AddrParseError;
use thiserror::Error;
use tonic::transport::Server;
use tracing::{event, Level};

/// App manages the state of the whole application
/// including sub-services
pub struct App {}

impl App {
    /// starts all the services belonging to the grpc server
    /// including the health_service.
    pub async fn run_server(grpc_server_port: u16, logs_dir: &str) -> Result<(), AppError> {
        init_tracing(logs_dir);

        let (health_reporter, health_service) = tonic_health::server::health_reporter();
        let mut metrics_service = MetricsService::new(health_reporter.clone());

        let addr = format!("[::1]:{0}", grpc_server_port).parse()?;

        event!(Level::INFO, "starting grpc server");

        Server::builder()
            .add_service(health_service)
            .add_service(metrics_service.ingestion_server().await)
            .serve(addr)
            .await?;

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid port")]
    InvalidPort(#[from] AddrParseError),
    #[error("Could not start serving the grpc on port")]
    GrpcStartError(#[from] tonic::transport::Error),
}

fn init_tracing(logs_dir: &str) {
    let file_appender = tracing_appender::rolling::hourly(logs_dir, "grpc_server.log");
    tracing_subscriber::fmt().with_writer(file_appender).init();
}
