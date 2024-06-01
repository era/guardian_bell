use crate::server;
use proto::ingestion_server::{Ingestion, IngestionServer};
use proto::{PutRequest, PutResponse};
use tonic::{Request, Response, Status};
use tonic_health::server::HealthReporter;
use tracing::instrument;

pub mod proto {
    tonic::include_proto!("metrics_service");
    pub mod common {
        pub mod v1 {
            tonic::include_proto!("opentelemetry.proto.common.v1");
        }
    }
    pub mod resource {
        pub mod v1 {
            tonic::include_proto!("opentelemetry.proto.resource.v1");
        }
    }
    pub mod metrics {
        pub mod v1 {
            tonic::include_proto!("opentelemetry.proto.metrics.v1");
        }
    }
    pub mod collector {
        pub mod metrics {
            pub mod v1 {
                tonic::include_proto!("opentelemetry.proto.collector.metrics.v1");
            }
        }
    }
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
        self.health_reporter
            .set_serving::<IngestionServer<MetricsService>>()
            .await;
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

//TODO stop usign our mock ingestion grpc service and start using open telemetry
#[tonic::async_trait]
impl Ingestion for MetricsService {
    #[instrument]
    async fn put(&self, _req: Request<PutRequest>) -> Result<Response<PutResponse>, Status> {
        Ok(Response::new(PutResponse {}))
    }
}
