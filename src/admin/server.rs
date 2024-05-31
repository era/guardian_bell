use crate::server;
use proto::admin_server::{Admin, AdminServer};
use proto::{ShutdownRequest, ShutdownResponse};
use tokio::sync::mpsc::Sender;
use tonic::{Request, Response, Status};
use tonic_health::server::HealthReporter;
use tracing::instrument;

pub mod proto {
    tonic::include_proto!("admin_service");
}

#[derive(Clone, Debug)]
pub struct AdminService {
    health_reporter: HealthReporter,
    tx: Sender<bool>,
}

impl AdminService {
    pub fn new(health_reporter: HealthReporter, tx: Sender<bool>) -> Self {
        Self {
            health_reporter,
            tx,
        }
    }

    pub async fn admin_server(&mut self) -> AdminServer<AdminService> {
        //FIXME
        self.health_reporter
            .set_serving::<AdminServer<AdminService>>()
            .await; // set_not_serving for unhealthy
        AdminServer::new(self.clone())
    }
}

#[tonic::async_trait]
impl server::Administrable for AdminService {
    async fn shutdown(&mut self) -> Result<(), server::ShutdownError> {
        let mut health_reporter = self.health_reporter.clone();
        health_reporter
            .set_not_serving::<AdminServer<AdminService>>()
            .await;

        Ok(())
    }

    fn service_name(&self) -> &str {
        "AdminService"
    }
}

#[tonic::async_trait]
impl Admin for AdminService {
    #[instrument]
    async fn shutdown(
        &self,
        _req: Request<ShutdownRequest>,
    ) -> Result<Response<ShutdownResponse>, Status> {
        self.tx.send(true).await.unwrap(); //FIXME
        Ok(Response::new(ShutdownResponse {}))
    }
}
