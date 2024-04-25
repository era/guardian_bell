use proto::ingestion_server::{Ingestion, IngestionServer};
use proto::{PutRequest, PutResponse};
use tonic::{transport::Server, Request, Response, Status};
use tonic_health::server::HealthReporter;

pub mod proto {
    tonic::include_proto!("metrics_service");
}

pub struct MetricsService {
    health_reporter: HealthReporter,
}

impl MetricsService {
    pub fn new(health_reporter: HealthReporter) -> Self {
        Self { health_reporter }
    }

    pub async fn to_grpc(mut self) -> IngestionServer<MetricsService> {
        //FIXME
        self.health_reporter
            .set_serving::<IngestionServer<MetricsService>>()
            .await; // set_not_serving for unhealthy
        IngestionServer::new(self)
    }
}

#[tonic::async_trait]
impl Ingestion for MetricsService {
    async fn put(&self, req: Request<PutRequest>) -> Result<Response<PutResponse>, Status> {
        Ok(Response::new(PutResponse {}))
    }
}
