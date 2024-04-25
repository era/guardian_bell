use tonic::{transport::Server, Request, Response, Status};
use tonic_health::server::HealthReporter;

mod grpc;
mod metrics;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

    let addr = "[::1]:50051".parse().unwrap();
    let metrics_service = metrics::server::MetricsService::new(health_reporter.clone());

    Server::builder()
        .add_service(health_service)
        .add_service(metrics_service.to_grpc().await)
        .serve(addr)
        .await?;

    Ok(())
}
