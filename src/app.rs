use crate::admin::server::AdminService;
use crate::metrics::server::MetricsService;
use crate::server;
use std::net::AddrParseError;
use std::path::Path;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tonic::transport::Server;
use tracing::{event, Level};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid port")]
    InvalidPort(#[from] AddrParseError),
    #[error("Could not start serving the grpc on port")]
    GrpcStartError(#[from] tonic::transport::Error),
}

/// App manages the state of the whole application
/// including sub-services
pub struct App {}

impl App {
    /// starts all the services belonging to the grpc server
    /// including the health_service.
    pub async fn run_server(grpc_server_port: u16, logs_dir: &Path) -> Result<(), AppError> {
        init_tracing(logs_dir);

        // creates a channel to warn when server should shutdown
        let (tx, rx): (Sender<bool>, Receiver<bool>) = mpsc::channel(1);
        let (health_reporter, health_service) = tonic_health::server::health_reporter();

        let mut metrics_service = MetricsService::new(health_reporter.clone());
        let mut admin_service = AdminService::new(health_reporter.clone(), tx);
        let services: Vec<Box<dyn server::Administrable + Send>> = vec![
            Box::new(metrics_service.clone()),
            Box::new(admin_service.clone()),
        ];
        watch_server(rx, services);

        let addr = format!("[::1]:{0}", grpc_server_port).parse()?;

        event!(Level::INFO, "starting grpc server");

        Server::builder()
            .add_service(health_service)
            .add_service(metrics_service.ingestion_server().await)
            .add_service(admin_service.admin_server().await)
            .serve(addr)
            .await?;

        Ok(())
    }
}

fn watch_server(
    mut shutdown_channel: Receiver<bool>,
    services: Vec<Box<dyn server::Administrable + Send>>,
) {
    tokio::spawn(async move {
        let _ = shutdown_channel.recv().await;
        for mut service in services {
            let name = service.service_name().to_owned();
            match service.shutdown().await {
                Err(e) => event!(
                    Level::ERROR,
                    "error while shuting down service {:0} {:1}",
                    name,
                    e
                ),
                _ => event!(Level::INFO, "shutdown service {:0} succeeded", name),
            };
        }
    });
}

fn init_tracing(logs_dir: &Path) {
    let file_appender = tracing_appender::rolling::hourly(logs_dir, "grpc_server.log");
    tracing_subscriber::fmt().with_writer(file_appender).init();
}
