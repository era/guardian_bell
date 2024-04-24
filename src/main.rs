use tonic_health::server::HealthReporter;
use tonic::{transport::Server, Request, Response, Status};
use proto::ingestion_server::{Ingestion, IngestionServer};
use proto::{PutRequest, PutResponse};

pub mod proto {
    tonic::include_proto!("metrics_service");
}

#[derive(Default)]
pub struct MetricsService {}


#[tonic::async_trait]
impl Ingestion for MetricsService {
    async fn put(&self, req: Request<PutRequest>) -> Result<Response<PutResponse>, Status> {
        Ok(Response::new(PutResponse{}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_serving::<IngestionServer<MetricsService>>().await; // set_not_serving for unhealthy

    let addr = "[::1]:50051".parse().unwrap();
    let metrics_service = MetricsService::default();

    Server::builder()
        .add_service(health_service)
        .add_service(IngestionServer::new(metrics_service))
        .serve(addr)
        .await?;
    
    Ok(())
}