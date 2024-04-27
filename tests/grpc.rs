use temp_dir::TempDir;
use tokio::runtime::Handle;
use tokio::time::sleep;
use tokio::time::Duration;
use tonic_health::pb::health_client::HealthClient;

fn start_grpc_server() {
    Handle::current().spawn(async {
        let logs = TempDir::new().unwrap();
        guardian_bell::app::App::run_server(8080, logs.path())
            .await
            .unwrap();
    });
}

async fn setup() {
    start_grpc_server();
    //FIXME
    sleep(Duration::from_secs(3)).await;
}

#[tokio::test]
async fn health_service() {
    setup().await;
    let conn = tonic::transport::Endpoint::new("http://[::1]:8080")
        .unwrap()
        .connect()
        .await
        .unwrap();
    let client = HealthClient::new(conn);
    // TODO: test the health service
}
