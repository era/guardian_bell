use crate::server;
use proto::ingestion_server::{Ingestion, IngestionServer};
use proto::{PutRequest, PutResponse};
use tonic::{Request, Response, Status};
use tonic_health::server::HealthReporter;
use tracing::instrument;

pub mod proto {
    tonic::include_proto!("metrics_service");
}

#[derive(Clone, Debug)]
pub struct MetricsService {
    health_reporter: HealthReporter,
}

impl MetricsService {
    pub fn new(health_reporter: HealthReporter) -> Self {
        Self { health_reporter }
    }

    pub async fn ingestion_server(&mut self) -> IngestionServer<MetricsService> {
        //FIXME
        self.health_reporter
            .set_serving::<IngestionServer<MetricsService>>()
            .await; // set_not_serving for unhealthy
        IngestionServer::new(self.clone())
    }
}

#[tonic::async_trait]
impl server::Administrable for MetricsService {
    async fn shutdown(&mut self) -> Result<(), server::ShutdownError> {
        let mut health_reporter = self.health_reporter.clone();
        health_reporter
            .set_not_serving::<IngestionServer<MetricsService>>()
            .await;

        Ok(())
    }

    fn service_name(&self) -> &str {
        "MetricsService"
    }
}

#[tonic::async_trait]
impl Ingestion for MetricsService {
    #[instrument]
    async fn put(&self, _req: Request<PutRequest>) -> Result<Response<PutResponse>, Status> {
        Ok(Response::new(PutResponse {}))
    }
}
