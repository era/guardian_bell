use temp_dir::TempDir;
use tokio::runtime::Handle;
use tokio::time::sleep;
use tokio::time::Duration;
use tonic_health::pb::health_check_response::ServingStatus;
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
    sleep(Duration::from_secs(1)).await;
}

#[tokio::test]
async fn health_service() {
    setup().await;
    let conn = tonic::transport::Endpoint::new("http://[::1]:8080")
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut client = HealthClient::new(conn);
    let request = tonic::Request::new(tonic_health::pb::HealthCheckRequest { service: "".into() });
    let response = client.check(request).await.unwrap();
    let response = response.into_inner();

    assert_eq!(
        ServingStatus::Serving,
        ServingStatus::from_i32(response.status).unwrap()
    );
}

// TODO send a request to mark the node as not_serving (so we can shutdown it)
// Test if SercingStatus return not_serving
